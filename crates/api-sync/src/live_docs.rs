//! Live CRDT documents for shared notes.
//!
//! Each web-editable shared note may own a Yjs document that concurrent editors
//! mutate through a WebSocket relay. Durable state lives in Supabase as a
//! compacted state blob plus an ordered update log; every sync instance loads
//! that state on demand, appends the updates it relays, and polls the log so
//! peers connected to other Fly Machines converge without a shared broker. The
//! existing snapshot/revision path stays the durable fallback: clients flush the
//! CRDT document back into the snapshot through the regular web-edit route.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::{Mutex, mpsc};
use yrs::sync::{Message, SyncMessage};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

use crate::error::{Result, SyncError};
use crate::state::AppState;

pub(crate) const MAX_LIVE_UPDATE_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_LIVE_STATE_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const LIVE_TICKET_TTL: Duration = Duration::from_secs(60);
const LIVE_RPC_TIMEOUT: Duration = Duration::from_secs(5);
const LIVE_POLL_INTERVAL: Duration = Duration::from_millis(1500);
const LIVE_IDLE_UNLOAD: Duration = Duration::from_secs(30);
const LIVE_UPDATE_PAGE: i64 = 256;
const COMPACT_AFTER_UPDATES: u32 = 200;
const TICKET_DOMAIN: &[u8] = b"anarlog-live-ticket-v1:";
const MAX_LIVE_RPC_RESPONSE_BYTES: usize = MAX_LIVE_STATE_BYTES + 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LiveCapability {
    Editor,
    Viewer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LiveAccess {
    pub(crate) share_id: String,
    pub(crate) capability: LiveCapability,
    pub(crate) access_version: i64,
    pub(crate) content_revision: i64,
}

#[derive(Serialize, Deserialize)]
struct TicketClaims {
    share_id: String,
    user_id: String,
    capability: LiveCapability,
    content_revision: i64,
    expires_at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LiveTicket {
    pub(crate) share_id: String,
    pub(crate) user_id: String,
    pub(crate) capability: LiveCapability,
    pub(crate) content_revision: i64,
}

fn ticket_mac(secret: &str) -> Hmac<Sha256> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts service-role keys of any size");
    mac.update(TICKET_DOMAIN);
    mac
}

pub(crate) fn issue_ticket(
    secret: &str,
    user_id: &str,
    access: &LiveAccess,
    now_unix: i64,
) -> String {
    let claims = TicketClaims {
        share_id: access.share_id.clone(),
        user_id: user_id.to_string(),
        capability: access.capability,
        content_revision: access.content_revision,
        expires_at: now_unix + LIVE_TICKET_TTL.as_secs() as i64,
    };
    let payload =
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).expect("ticket claims serialize"));
    let mut mac = ticket_mac(secret);
    mac.update(payload.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!("{payload}.{signature}")
}

pub(crate) fn verify_ticket(secret: &str, ticket: &str, now_unix: i64) -> Option<LiveTicket> {
    if ticket.len() > 2048 {
        return None;
    }
    let (payload, signature) = ticket.split_once('.')?;
    let signature = URL_SAFE_NO_PAD.decode(signature).ok()?;
    let mut mac = ticket_mac(secret);
    mac.update(payload.as_bytes());
    mac.verify_slice(&signature).ok()?;
    let claims: TicketClaims =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).ok()?).ok()?;
    if claims.expires_at <= now_unix {
        return None;
    }
    Some(LiveTicket {
        share_id: claims.share_id,
        user_id: claims.user_id,
        capability: claims.capability,
        content_revision: claims.content_revision,
    })
}

pub(crate) fn encode_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from_digit(u32::from(byte >> 4), 16).unwrap());
        encoded.push(char::from_digit(u32::from(byte & 0x0f), 16).unwrap());
    }
    encoded
}

pub(crate) fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    value
        .as_bytes()
        .chunks(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16)?;
            let low = (pair[1] as char).to_digit(16)?;
            Some((high * 16 + low) as u8)
        })
        .collect()
}

#[derive(Deserialize)]
struct PostgrestError {
    code: String,
}

async fn rpc<RequestBody, Row>(
    state: &AppState,
    function: &str,
    request: &RequestBody,
) -> Result<Vec<Row>>
where
    RequestBody: Serialize + ?Sized,
    Row: serde::de::DeserializeOwned,
{
    let mut response = state
        .client
        .post(format!(
            "{}/rest/v1/rpc/{function}",
            state.config.supabase_url
        ))
        .header("apikey", &state.config.supabase_service_role_key)
        .bearer_auth(&state.config.supabase_service_role_key)
        .timeout(LIVE_RPC_TIMEOUT)
        .json(request)
        .send()
        .await
        .map_err(|error| {
            tracing::warn!(%error, function, "live document RPC request failed");
            SyncError::SnapshotServiceUnavailable
        })?;
    let status = response.status();
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| {
        tracing::warn!(%error, function, "live document RPC response could not be read");
        SyncError::SnapshotServiceUnavailable
    })? {
        if bytes.len().saturating_add(chunk.len()) > MAX_LIVE_RPC_RESPONSE_BYTES {
            tracing::warn!(%status, function, "live document RPC response was too large");
            return Err(SyncError::SnapshotServiceUnavailable);
        }
        bytes.extend_from_slice(&chunk);
    }
    if !status.is_success() {
        let code = serde_json::from_slice::<PostgrestError>(&bytes)
            .ok()
            .map(|error| error.code);
        tracing::warn!(%status, ?code, function, "live document RPC was rejected");
        return match code.as_deref() {
            Some("42501") => Err(SyncError::SnapshotPublicationForbidden),
            Some("22023") => Err(SyncError::BadRequest(
                "Live document update is invalid".to_string(),
            )),
            _ => Err(SyncError::SnapshotServiceUnavailable),
        };
    }
    serde_json::from_slice::<Vec<Row>>(&bytes).map_err(|error| {
        tracing::warn!(%error, function, "live document RPC response was invalid");
        SyncError::SnapshotServiceUnavailable
    })
}

#[derive(Serialize)]
struct ResolveAccessRequest<'a> {
    p_share_id: &'a str,
    p_actor_user_id: &'a str,
}

#[derive(Deserialize)]
struct ResolveAccessRow {
    share_id: String,
    capability: String,
    access_version: i64,
    content_revision: i64,
}

pub(crate) async fn resolve_access(
    state: &AppState,
    share_id: &str,
    user_id: &str,
) -> Result<LiveAccess> {
    let mut rows: Vec<ResolveAccessRow> = rpc(
        state,
        "resolve_session_share_live_access",
        &ResolveAccessRequest {
            p_share_id: share_id,
            p_actor_user_id: user_id,
        },
    )
    .await?;
    let Some(row) = rows.pop() else {
        return Err(SyncError::SharedNoteNotFound);
    };
    if !rows.is_empty() || row.share_id != share_id || row.access_version < 1 {
        return Err(SyncError::SnapshotServiceUnavailable);
    }
    let capability = match row.capability.as_str() {
        "editor" => LiveCapability::Editor,
        "viewer" => LiveCapability::Viewer,
        _ => return Err(SyncError::SnapshotServiceUnavailable),
    };
    Ok(LiveAccess {
        share_id: row.share_id,
        capability,
        access_version: row.access_version,
        content_revision: row.content_revision,
    })
}

#[derive(Serialize)]
struct ReadDocumentRequest<'a> {
    p_share_id: &'a str,
}

#[derive(Deserialize)]
struct ReadDocumentRow {
    state_hex: Option<String>,
    compacted_through_seq: i64,
    updates: Vec<UpdateRow>,
}

#[derive(Deserialize)]
struct UpdateRow {
    seq: i64,
    update_hex: String,
}

#[derive(Serialize)]
struct ReadUpdatesRequest<'a> {
    p_share_id: &'a str,
    p_after_seq: i64,
    p_limit: i64,
}

#[derive(Serialize)]
struct AppendUpdateRequest<'a> {
    p_share_id: &'a str,
    p_actor_user_id: &'a str,
    p_update_hex: &'a str,
    p_require_empty: bool,
}

#[derive(Deserialize)]
struct AppendUpdateRow {
    outcome: String,
    seq: Option<i64>,
}

#[derive(Serialize)]
struct CompactRequest<'a> {
    p_share_id: &'a str,
    p_state_hex: &'a str,
    p_through_seq: i64,
}

#[derive(Deserialize)]
struct CompactRow {
    compacted_through_seq: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AppendOutcome {
    Appended(i64),
    NotEmpty,
}

async fn append_update(
    state: &AppState,
    share_id: &str,
    user_id: &str,
    update: &[u8],
    require_empty: bool,
) -> Result<AppendOutcome> {
    let mut rows: Vec<AppendUpdateRow> = rpc(
        state,
        "append_session_share_live_update",
        &AppendUpdateRequest {
            p_share_id: share_id,
            p_actor_user_id: user_id,
            p_update_hex: &encode_hex(update),
            p_require_empty: require_empty,
        },
    )
    .await?;
    let Some(row) = rows.pop() else {
        return Err(SyncError::SnapshotServiceUnavailable);
    };
    match (row.outcome.as_str(), row.seq) {
        ("appended", Some(seq)) if seq > 0 => Ok(AppendOutcome::Appended(seq)),
        ("not_empty", None) => Ok(AppendOutcome::NotEmpty),
        _ => Err(SyncError::SnapshotServiceUnavailable),
    }
}

pub(crate) type PeerId = u64;

/// A frame queued for one peer's socket writer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Outbound {
    Binary(Vec<u8>),
    Control(String),
}

/// Frames a peer may have queued before it is considered stalled and dropped;
/// a dropped peer reconnects and resyncs from the authoritative document.
pub(crate) const PEER_OUTBOUND_CAPACITY: usize = 256;

pub(crate) type PeerSender = mpsc::Sender<Outbound>;

struct LiveDocInner {
    doc: Doc,
    polled_through: i64,
    updates_since_compaction: u32,
    peers: HashMap<PeerId, PeerSender>,
    next_peer: PeerId,
    poller_running: bool,
}

impl LiveDocInner {
    fn is_empty(&self) -> bool {
        self.doc.transact().state_vector().is_empty()
    }

    fn apply(&mut self, update: &[u8]) -> Result<bool> {
        let update = Update::decode_v1(update)
            .map_err(|_| SyncError::BadRequest("Live document update is invalid".to_string()))?;
        let before = self.doc.transact().state_vector();
        let mut txn = self.doc.transact_mut();
        txn.apply_update(update)
            .map_err(|_| SyncError::BadRequest("Live document update is invalid".to_string()))?;
        let deleted = !txn.delete_set().is_empty();
        drop(txn);
        Ok(deleted || before != self.doc.transact().state_vector())
    }

    fn broadcast(&mut self, from: Option<PeerId>, frame: &[u8]) {
        self.peers.retain(|peer_id, sender| {
            if Some(*peer_id) == from {
                return true;
            }
            sender.try_send(Outbound::Binary(frame.to_vec())).is_ok()
        });
    }
}

pub(crate) struct LiveDoc {
    share_id: String,
    inner: Mutex<LiveDocInner>,
}

impl LiveDoc {
    pub(crate) async fn send_to(
        &self,
        peer_id: PeerId,
        frame: Outbound,
    ) -> std::result::Result<(), ()> {
        let mut inner = self.inner.lock().await;
        let sent = inner
            .peers
            .get(&peer_id)
            .is_some_and(|sender| sender.try_send(frame).is_ok());
        if !sent {
            // A peer that cannot take a direct reply (stalled or gone) is
            // dropped rather than left with a partial protocol exchange.
            inner.peers.remove(&peer_id);
        }
        sent.then_some(()).ok_or(())
    }
}

#[derive(Clone, Default)]
pub struct LiveDocs {
    docs: Arc<Mutex<HashMap<String, Arc<LiveDoc>>>>,
}

pub(crate) fn encode_frame(message: &Message) -> Vec<u8> {
    message.encode_v1()
}

pub(crate) fn decode_frame(frame: &[u8]) -> Result<Message> {
    Message::decode_v1(frame)
        .map_err(|_| SyncError::BadRequest("Live document message is invalid".to_string()))
}

/// What a connected peer receives after the handshake.
#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum ControlMessage {
    #[serde(rename_all = "camelCase")]
    Ready {
        capability: LiveCapability,
        content_revision: i64,
        seed_required: bool,
    },
    Reset,
}

impl LiveDocs {
    async fn load(&self, state: &AppState, share_id: &str) -> Result<Arc<LiveDoc>> {
        let existing = self.docs.lock().await.get(share_id).cloned();
        if let Some(doc) = existing {
            return Ok(doc);
        }
        let mut rows: Vec<ReadDocumentRow> = rpc(
            state,
            "read_session_share_live_document",
            &ReadDocumentRequest {
                p_share_id: share_id,
            },
        )
        .await?;
        let Some(row) = rows.pop() else {
            return Err(SyncError::SnapshotServiceUnavailable);
        };
        let doc = Doc::new();
        let mut polled_through = row.compacted_through_seq;
        {
            let mut txn = doc.transact_mut();
            if let Some(state_hex) = row.state_hex.as_deref() {
                let bytes = decode_hex(state_hex).ok_or(SyncError::SnapshotServiceUnavailable)?;
                let update =
                    Update::decode_v1(&bytes).map_err(|_| SyncError::SnapshotServiceUnavailable)?;
                txn.apply_update(update)
                    .map_err(|_| SyncError::SnapshotServiceUnavailable)?;
            }
            for update in row.updates {
                let bytes =
                    decode_hex(&update.update_hex).ok_or(SyncError::SnapshotServiceUnavailable)?;
                let update_value =
                    Update::decode_v1(&bytes).map_err(|_| SyncError::SnapshotServiceUnavailable)?;
                txn.apply_update(update_value)
                    .map_err(|_| SyncError::SnapshotServiceUnavailable)?;
                polled_through = polled_through.max(update.seq);
            }
        }
        let live = Arc::new(LiveDoc {
            share_id: share_id.to_string(),
            inner: Mutex::new(LiveDocInner {
                doc,
                polled_through,
                updates_since_compaction: u32::try_from(polled_through - row.compacted_through_seq)
                    .unwrap_or(u32::MAX),
                peers: HashMap::new(),
                next_peer: 1,
                poller_running: false,
            }),
        });
        // The bootstrap RPC returns only the first page of uncompacted updates;
        // page through the rest so long logs never need a single oversized read.
        self.refresh(state, &live).await?;
        let mut docs = self.docs.lock().await;
        Ok(Arc::clone(docs.entry(share_id.to_string()).or_insert(live)))
    }

    /// Registers a peer, returning its id, the handshake frames it must receive
    /// first (sync step 1 + step 2) and whether the document still needs seeding.
    pub(crate) async fn join(
        &self,
        state: &AppState,
        share_id: &str,
        sender: PeerSender,
    ) -> Result<(Arc<LiveDoc>, PeerId, Vec<Vec<u8>>, bool)> {
        let live = self.load(state, share_id).await?;
        let (peer_id, frames, seed_required, start_poller) = {
            let mut inner = live.inner.lock().await;
            let peer_id = inner.next_peer;
            inner.next_peer += 1;
            inner.peers.insert(peer_id, sender);
            let txn = inner.doc.transact();
            let frames = vec![
                encode_frame(&Message::Sync(SyncMessage::SyncStep1(txn.state_vector()))),
                encode_frame(&Message::Sync(SyncMessage::SyncStep2(
                    txn.encode_state_as_update_v1(&StateVector::default()),
                ))),
            ];
            let seed_required = txn.state_vector().is_empty();
            drop(txn);
            let start_poller = !inner.poller_running;
            inner.poller_running = true;
            (peer_id, frames, seed_required, start_poller)
        };
        if start_poller {
            tokio::spawn(poll_loop(self.clone(), state.clone(), Arc::clone(&live)));
        }
        Ok((live, peer_id, frames, seed_required))
    }

    pub(crate) async fn leave(&self, live: &LiveDoc, peer_id: PeerId) {
        live.inner.lock().await.peers.remove(&peer_id);
    }

    /// Handles one inbound frame from `peer_id`. Returns frames to send back to
    /// that peer only.
    pub(crate) async fn handle_frame(
        &self,
        state: &AppState,
        live: &LiveDoc,
        peer_id: PeerId,
        user_id: &str,
        capability: LiveCapability,
        frame: &[u8],
    ) -> Result<Vec<Outbound>> {
        if frame.len() > MAX_LIVE_UPDATE_BYTES {
            return Err(SyncError::BadRequest(
                "Live document update is too large".to_string(),
            ));
        }
        match decode_frame(frame)? {
            Message::Sync(SyncMessage::SyncStep1(state_vector)) => {
                let inner = live.inner.lock().await;
                let txn = inner.doc.transact();
                Ok(vec![Outbound::Binary(encode_frame(&Message::Sync(
                    SyncMessage::SyncStep2(txn.encode_diff_v1(&state_vector)),
                )))])
            }
            Message::Sync(SyncMessage::SyncStep2(update)) => {
                if capability != LiveCapability::Editor {
                    // Viewers answer our sync step 1 with a step 2 that carries
                    // nothing we accept; drop it instead of treating it as an edit.
                    return Ok(Vec::new());
                }
                self.apply_from_peer(state, live, peer_id, user_id, &update)
                    .await
            }
            Message::Sync(SyncMessage::Update(update)) => {
                if capability != LiveCapability::Editor {
                    return Err(SyncError::SnapshotPublicationForbidden);
                }
                self.apply_from_peer(state, live, peer_id, user_id, &update)
                    .await
            }
            Message::Awareness(_) | Message::AwarenessQuery => {
                // Presence is a later phase; frames are ignored rather than relayed.
                Ok(Vec::new())
            }
            Message::Auth(_) | Message::Custom(_, _) => Err(SyncError::BadRequest(
                "Live document message is invalid".to_string(),
            )),
        }
    }

    async fn apply_from_peer(
        &self,
        state: &AppState,
        live: &LiveDoc,
        peer_id: PeerId,
        user_id: &str,
        update: &[u8],
    ) -> Result<Vec<Outbound>> {
        Update::decode_v1(update)
            .map_err(|_| SyncError::BadRequest("Live document update is invalid".to_string()))?;
        let require_empty = live.inner.lock().await.is_empty();
        match append_update(state, &live.share_id, user_id, update, require_empty).await? {
            AppendOutcome::Appended(_) => {}
            AppendOutcome::NotEmpty => {
                // Another instance seeded first; the peer must rebuild from the
                // authoritative state instead of merging its local seed.
                self.refresh(state, live).await?;
                let inner = live.inner.lock().await;
                let txn = inner.doc.transact();
                return Ok(vec![
                    Outbound::Control(control_frame(&ControlMessage::Reset)),
                    Outbound::Binary(encode_frame(&Message::Sync(SyncMessage::SyncStep2(
                        txn.encode_state_as_update_v1(&StateVector::default()),
                    )))),
                ]);
            }
        }
        let mut inner = live.inner.lock().await;
        let changed = inner.apply(update)?;
        if changed {
            let frame = encode_frame(&Message::Sync(SyncMessage::Update(update.to_vec())));
            inner.broadcast(Some(peer_id), &frame);
        }
        Ok(Vec::new())
    }

    /// Pulls updates appended by any instance (including this one) since the
    /// last poll and fans new content out to local peers.
    pub(crate) async fn refresh(&self, state: &AppState, live: &LiveDoc) -> Result<()> {
        loop {
            let after_seq = live.inner.lock().await.polled_through;
            let rows: Vec<UpdateRow> = rpc(
                state,
                "read_session_share_live_updates",
                &ReadUpdatesRequest {
                    p_share_id: &live.share_id,
                    p_after_seq: after_seq,
                    p_limit: LIVE_UPDATE_PAGE,
                },
            )
            .await?;
            let count = rows.len() as i64;
            if rows.first().is_some_and(|row| row.seq > after_seq + 1) {
                // Another instance compacted past our cursor; the missing rows
                // now live only in the compacted state.
                self.merge_compacted_state(state, live).await?;
            }
            let mut inner = live.inner.lock().await;
            for row in rows {
                let bytes =
                    decode_hex(&row.update_hex).ok_or(SyncError::SnapshotServiceUnavailable)?;
                if inner.apply(&bytes)? {
                    let frame = encode_frame(&Message::Sync(SyncMessage::Update(bytes)));
                    inner.broadcast(None, &frame);
                }
                inner.polled_through = inner.polled_through.max(row.seq);
                inner.updates_since_compaction = inner.updates_since_compaction.saturating_add(1);
            }
            if count < LIVE_UPDATE_PAGE {
                return Ok(());
            }
        }
    }

    async fn merge_compacted_state(&self, state: &AppState, live: &LiveDoc) -> Result<()> {
        let mut rows: Vec<ReadDocumentRow> = rpc(
            state,
            "read_session_share_live_document",
            &ReadDocumentRequest {
                p_share_id: &live.share_id,
            },
        )
        .await?;
        let Some(row) = rows.pop() else {
            return Err(SyncError::SnapshotServiceUnavailable);
        };
        let Some(state_hex) = row.state_hex.as_deref() else {
            return Ok(());
        };
        let bytes = decode_hex(state_hex).ok_or(SyncError::SnapshotServiceUnavailable)?;
        let mut inner = live.inner.lock().await;
        let before = inner.doc.transact().state_vector();
        if inner.apply(&bytes)? {
            let diff = inner.doc.transact().encode_diff_v1(&before);
            let frame = encode_frame(&Message::Sync(SyncMessage::Update(diff)));
            inner.broadcast(None, &frame);
        }
        inner.polled_through = inner.polled_through.max(row.compacted_through_seq);
        Ok(())
    }

    async fn maybe_compact(&self, state: &AppState, live: &LiveDoc) -> Result<()> {
        let (state_bytes, through_seq) = {
            let mut inner = live.inner.lock().await;
            if inner.updates_since_compaction < COMPACT_AFTER_UPDATES {
                return Ok(());
            }
            inner.updates_since_compaction = 0;
            let txn = inner.doc.transact();
            (
                txn.encode_state_as_update_v1(&StateVector::default()),
                inner.polled_through,
            )
        };
        if state_bytes.len() > MAX_LIVE_STATE_BYTES || through_seq == 0 {
            return Ok(());
        }
        let rows: Vec<CompactRow> = rpc(
            state,
            "compact_session_share_live_document",
            &CompactRequest {
                p_share_id: &live.share_id,
                p_state_hex: &encode_hex(&state_bytes),
                p_through_seq: through_seq,
            },
        )
        .await?;
        if rows
            .first()
            .is_none_or(|row| row.compacted_through_seq < through_seq)
        {
            tracing::debug!(share_id = %live.share_id, "live document compaction was superseded");
        }
        Ok(())
    }

    async fn unload_if_idle(&self, live: &Arc<LiveDoc>) -> bool {
        let mut docs = self.docs.lock().await;
        let mut inner = live.inner.lock().await;
        if !inner.peers.is_empty() {
            return false;
        }
        inner.poller_running = false;
        if docs
            .get(&live.share_id)
            .is_some_and(|current| Arc::ptr_eq(current, live))
        {
            docs.remove(&live.share_id);
        }
        true
    }
}

async fn poll_loop(docs: LiveDocs, state: AppState, live: Arc<LiveDoc>) {
    let mut idle_since: Option<tokio::time::Instant> = None;
    loop {
        tokio::time::sleep(LIVE_POLL_INTERVAL).await;
        if let Err(error) = docs.refresh(&state, &live).await {
            tracing::warn!(%error, share_id = %live.share_id, "live document poll failed");
        } else if let Err(error) = docs.maybe_compact(&state, &live).await {
            tracing::warn!(%error, share_id = %live.share_id, "live document compaction failed");
        }
        let has_peers = !live.inner.lock().await.peers.is_empty();
        if has_peers {
            idle_since = None;
            continue;
        }
        let since = *idle_since.get_or_insert_with(tokio::time::Instant::now);
        if since.elapsed() >= LIVE_IDLE_UNLOAD && docs.unload_if_idle(&live).await {
            return;
        }
    }
}

pub(crate) fn control_frame(message: &ControlMessage) -> String {
    serde_json::to_string(message).expect("control serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use yrs::{GetString, Text};

    fn access() -> LiveAccess {
        LiveAccess {
            share_id: "11111111-1111-4111-8111-111111111111".to_string(),
            capability: LiveCapability::Editor,
            access_version: 1,
            content_revision: 3,
        }
    }

    #[test]
    fn ticket_round_trips_and_expires() {
        let ticket = issue_ticket("secret", "user-1", &access(), 1_000);
        let verified = verify_ticket("secret", &ticket, 1_030).expect("ticket verifies");
        assert_eq!(verified.share_id, access().share_id);
        assert_eq!(verified.user_id, "user-1");
        assert_eq!(verified.capability, LiveCapability::Editor);
        assert_eq!(verified.content_revision, 3);
        assert!(verify_ticket("secret", &ticket, 1_060).is_none());
        assert!(verify_ticket("other", &ticket, 1_030).is_none());
    }

    #[test]
    fn tampered_tickets_are_rejected() {
        let ticket = issue_ticket("secret", "user-1", &access(), 1_000);
        let (payload, signature) = ticket.split_once('.').unwrap();
        let mut claims: serde_json::Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap();
        claims["capability"] = serde_json::Value::String("editor".to_string());
        claims["user_id"] = serde_json::Value::String("user-2".to_string());
        let forged = format!(
            "{}.{signature}",
            URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).unwrap())
        );
        assert!(verify_ticket("secret", &forged, 1_030).is_none());
        assert!(verify_ticket("secret", "not-a-ticket", 1_030).is_none());
    }

    #[test]
    fn hex_round_trips() {
        let bytes = vec![0u8, 1, 15, 16, 170, 255];
        assert_eq!(encode_hex(&bytes), "00010f10aaff");
        assert_eq!(decode_hex("00010f10aaff"), Some(bytes));
        assert_eq!(decode_hex("abc"), None);
        assert_eq!(decode_hex("zz"), None);
    }

    #[test]
    fn sync_frames_round_trip_through_yrs_codec() {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("t");
        text.push(&mut doc.transact_mut(), "hello");
        let update = doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default());
        let frame = encode_frame(&Message::Sync(SyncMessage::Update(update.clone())));
        match decode_frame(&frame).unwrap() {
            Message::Sync(SyncMessage::Update(decoded)) => assert_eq!(decoded, update),
            other => panic!("unexpected message {other:?}"),
        }
        let other = Doc::new();
        other
            .transact_mut()
            .apply_update(Update::decode_v1(&update).unwrap())
            .unwrap();
        assert_eq!(
            other.get_or_insert_text("t").get_string(&other.transact()),
            "hello"
        );
    }
}
