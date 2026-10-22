use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use anlg_db_reactive::QueryEventSink;

#[uniffi::export(with_foreign)]
pub trait QueryEventListener: Send + Sync {
    fn on_result(&self, rows_json: String);
    fn on_error(&self, message: String);
}

enum QueryEvent {
    Result(String),
    Error(String),
}

#[derive(Default)]
struct PendingEvent {
    event: Option<QueryEvent>,
    retired: bool,
}

struct ListenerState {
    listener: Arc<dyn QueryEventListener>,
    pending: Mutex<PendingEvent>,
}

impl ListenerState {
    fn deliver(&self, event: QueryEvent) {
        match event {
            QueryEvent::Result(rows_json) => self.listener.on_result(rows_json),
            QueryEvent::Error(message) => self.listener.on_error(message),
        }
    }

    fn deliver_pending(&self) {
        let event = self.pending.lock().unwrap().event.take();
        if let Some(event) = event {
            self.deliver(event);
        }
    }
}

/// Foreign listeners block their caller until the JS thread runs them, so
/// events raised off the subscribing thread are handed to a dedicated thread
/// instead of holding runtime workers and live-query state hostage to JS.
/// Only the latest undelivered event per listener is kept.
pub(crate) struct ListenerDelivery {
    sender: mpsc::Sender<Arc<ListenerState>>,
}

impl ListenerDelivery {
    pub(crate) fn spawn() -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel::<Arc<ListenerState>>();
        std::thread::Builder::new()
            .name("mobile-bridge-live-query".to_string())
            .spawn(move || {
                for state in receiver {
                    state.deliver_pending();
                }
            })?;
        Ok(Self { sender })
    }

    pub(crate) fn sink(&self, listener: Arc<dyn QueryEventListener>) -> ListenerSink {
        ListenerSink {
            state: Arc::new(ListenerState {
                listener,
                pending: Mutex::new(PendingEvent::default()),
            }),
            owner: std::thread::current().id(),
            sender: self.sender.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ListenerSink {
    state: Arc<ListenerState>,
    owner: ThreadId,
    sender: mpsc::Sender<Arc<ListenerState>>,
}

impl ListenerSink {
    pub(crate) fn retire(&self) {
        let mut pending = self.state.pending.lock().unwrap();
        pending.retired = true;
        pending.event = None;
    }

    fn send(&self, event: QueryEvent) -> std::result::Result<(), String> {
        if std::thread::current().id() == self.owner {
            self.state.deliver(event);
            return Ok(());
        }
        let mut pending = self.state.pending.lock().unwrap();
        if pending.retired {
            return Ok(());
        }
        if pending.event.replace(event).is_some() {
            return Ok(());
        }
        drop(pending);
        self.sender
            .send(Arc::clone(&self.state))
            .map_err(|_| "live query delivery stopped".to_string())
    }
}

impl QueryEventSink for ListenerSink {
    fn send_result(&self, rows: Vec<serde_json::Value>) -> std::result::Result<(), String> {
        let rows_json = serde_json::to_string(&rows).map_err(|error| error.to_string())?;
        self.send(QueryEvent::Result(rows_json))
    }

    fn send_error(&self, error: String) -> std::result::Result<(), String> {
        self.send(QueryEvent::Error(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct GatedListener {
        events: Mutex<Vec<String>>,
        started: mpsc::SyncSender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }

    impl QueryEventListener for GatedListener {
        fn on_result(&self, rows_json: String) {
            self.events.lock().unwrap().push(rows_json);
            let _ = self.started.try_send(());
            let _ = self
                .release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5));
        }

        fn on_error(&self, message: String) {
            self.on_result(message);
        }
    }

    fn rows(value: &str) -> Vec<serde_json::Value> {
        vec![serde_json::json!(value)]
    }

    #[test]
    fn off_owner_events_coalesce_and_stop_after_retire() {
        let (started_tx, started_rx) = mpsc::sync_channel(8);
        let (release_tx, release_rx) = mpsc::channel();
        let listener = Arc::new(GatedListener {
            events: Mutex::new(Vec::new()),
            started: started_tx,
            release: Mutex::new(release_rx),
        });
        let delivery = ListenerDelivery::spawn().unwrap();
        let sink = delivery.sink(listener.clone());

        let worker = sink.clone();
        std::thread::spawn(move || worker.send_result(rows("a")).unwrap())
            .join()
            .unwrap();
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();

        let worker = sink.clone();
        std::thread::spawn(move || {
            for value in ["b", "c", "d"] {
                worker.send_result(rows(value)).unwrap();
            }
        })
        .join()
        .unwrap();
        release_tx.send(()).unwrap();
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();

        sink.retire();
        let worker = sink.clone();
        std::thread::spawn(move || worker.send_result(rows("e")).unwrap())
            .join()
            .unwrap();
        release_tx.send(()).unwrap();
        assert!(started_rx.recv_timeout(Duration::from_millis(200)).is_err());

        assert_eq!(
            *listener.events.lock().unwrap(),
            vec![r#"["a"]"#.to_string(), r#"["d"]"#.to_string()]
        );
    }
}
