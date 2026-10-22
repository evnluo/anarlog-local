import { act, cleanup, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  signIn: vi.fn(),
  dismissToast: vi.fn(),
  openNew: vi.fn(),
  updateSettingsTabState: vi.fn(),
  clearDevtoolsPreview: vi.fn(),
  message: vi.fn(),
  error: vi.fn(),
  warning: vi.fn(),
  loading: vi.fn(),
  dismiss: vi.fn(),
  dismissedToastIds: new Set<string>(),
  sessionMode: "inactive",
  live: {
    status: "inactive" as "inactive" | "active" | "finalizing",
    sessionId: null as string | null,
  },
  currentTab: {
    type: "empty",
  } as {
    type: string;
    id?: string;
    state?: { tab?: string; view?: { type: string } };
  },
  config: {
    current_llm_provider: "local" as string | null,
    current_llm_model: "model" as string | null,
    current_stt_provider: "local" as string | null,
    current_stt_model: "model" as string | null,
  },
  notifications: {
    hasActiveDownload: false,
    downloadingModel: null as string | null,
    activeDownloads: [] as Array<{
      model: string;
      displayName: string;
      progress: number;
      isStarting?: boolean;
    }>,
    localSttStatus: null as null | "loading" | "unreachable",
    isLocalSttModel: false,
  },
  update: {
    status: null as null | "available" | "downloading" | "ready" | "failed",
    version: null as string | null,
    progress: null as number | null,
    errorMessage: null as string | null,
    downloadStarting: false,
    installing: false,
    downloadUpdate: vi.fn(),
    installUpdate: vi.fn(),
  },
}));

vi.mock("@anlg/ui/components/ui/toast", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("@anlg/ui/components/ui/toast")>();
  return {
    ...actual,
    toast: {
      message: mocks.message,
      error: mocks.error,
      warning: mocks.warning,
      loading: mocks.loading,
      dismiss: mocks.dismiss,
    },
  };
});

vi.mock("~/auth", () => ({
  useAuth: () => ({ session: null, signIn: mocks.signIn }),
}));

vi.mock("~/auth/cloudsync-progress", () => ({
  useCloudsyncInitialSyncProgress: () => ({ state: "idle" }),
}));

vi.mock("~/contexts/notifications", () => ({
  useNotifications: () => mocks.notifications,
}));

vi.mock("~/main/update-banner", () => ({
  useDesktopUpdateControl: () => mocks.update,
}));

vi.mock("~/shared/config", () => ({
  useConfigValues: () => mocks.config,
}));

vi.mock("~/store/zustand/devtools-toast-preview", () => ({
  useDevtoolsToastPreview: (
    selector: (state: { preview: null; clearPreview: () => void }) => unknown,
  ) =>
    selector({
      preview: null,
      clearPreview: mocks.clearDevtoolsPreview,
    }),
}));

vi.mock("~/store/zustand/tabs", () => ({
  useTabs: (
    selector: (state: {
      currentTab: typeof mocks.currentTab;
      openNew: () => void;
      updateSettingsTabState: () => void;
    }) => unknown,
  ) =>
    selector({
      currentTab: mocks.currentTab,
      openNew: mocks.openNew,
      updateSettingsTabState: mocks.updateSettingsTabState,
    }),
}));

vi.mock("~/stt/capabilities", () => ({
  isConfiguredSttModel: () => true,
  isAnarlogCloudSttModel: () => false,
}));

vi.mock("~/stt/contexts", () => ({
  useListener: (
    selector: (state: {
      getSessionMode: () => string;
      live: typeof mocks.live;
    }) => unknown,
  ) => selector({ getSessionMode: () => mocks.sessionMode, live: mocks.live }),
}));

vi.mock("./useDismissedToasts", () => ({
  useDismissedToasts: () => ({
    dismissToast: mocks.dismissToast,
    isDismissed: (id: string) => mocks.dismissedToastIds.has(id),
  }),
}));

import { ToastNotifications } from "./index";

const storedValues = new Map<string, string>();
const localStorageMock = {
  getItem: (key: string) => storedValues.get(key) ?? null,
  setItem: (key: string, value: string) => storedValues.set(key, value),
  removeItem: (key: string) => storedValues.delete(key),
  clear: () => storedValues.clear(),
};

describe("ToastNotifications", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.stubGlobal("localStorage", localStorageMock);
    mocks.signIn.mockClear();
    mocks.dismissToast.mockClear();
    mocks.message.mockClear();
    mocks.error.mockClear();
    mocks.warning.mockClear();
    mocks.loading.mockClear();
    mocks.dismiss.mockClear();
    mocks.dismissedToastIds.clear();
    localStorage.clear();
    mocks.live = { status: "inactive", sessionId: null };
    mocks.openNew.mockClear();
    mocks.updateSettingsTabState.mockClear();
    mocks.currentTab = { type: "empty" };
    mocks.config.current_llm_provider = "local";
    mocks.config.current_llm_model = "model";
    mocks.config.current_stt_provider = "local";
    mocks.config.current_stt_model = "model";
    mocks.notifications.hasActiveDownload = false;
    mocks.notifications.downloadingModel = null;
    mocks.notifications.activeDownloads = [];
    mocks.notifications.localSttStatus = null;
    mocks.notifications.isLocalSttModel = false;
    mocks.update.status = null;
    mocks.update.version = null;
    mocks.update.progress = null;
    mocks.update.errorMessage = null;
    mocks.update.downloadStarting = false;
    mocks.update.installing = false;
    mocks.update.downloadUpdate.mockClear();
    mocks.update.installUpdate.mockClear();
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it("lets users dismiss a model download toast until the next download", () => {
    mocks.notifications.hasActiveDownload = true;
    mocks.notifications.downloadingModel = "apple-speech";
    mocks.notifications.activeDownloads = [
      { model: "apple-speech", displayName: "apple-speech", progress: 0 },
    ];

    const view = render(<ToastNotifications />);
    act(() => vi.advanceTimersByTime(500));

    const options = mocks.loading.mock.calls[0][1];
    expect(options.closeButton).toBe(true);
    act(() => options.onDismiss());

    mocks.loading.mockClear();
    view.rerender(<ToastNotifications />);
    expect(mocks.loading).not.toHaveBeenCalled();

    mocks.notifications.hasActiveDownload = false;
    mocks.notifications.downloadingModel = null;
    mocks.notifications.activeDownloads = [];
    view.rerender(<ToastNotifications />);

    mocks.notifications.hasActiveDownload = true;
    mocks.notifications.downloadingModel = "apple-speech";
    mocks.notifications.activeDownloads = [
      { model: "apple-speech", displayName: "apple-speech", progress: 12 },
    ];
    mocks.loading.mockClear();
    view.rerender(<ToastNotifications />);

    expect(mocks.loading).toHaveBeenCalledWith(
      "Downloading apple-speech",
      expect.objectContaining({
        id: "downloading-model",
        closeButton: true,
      }),
    );
  });

  it("updates the model download toast from starting to download progress", () => {
    mocks.notifications.hasActiveDownload = true;
    mocks.notifications.downloadingModel = "Apple Speech";
    mocks.notifications.activeDownloads = [
      {
        model: "apple-speech",
        displayName: "Apple Speech",
        progress: 0,
        isStarting: true,
      },
    ];
    const view = render(<ToastNotifications />);
    act(() => vi.advanceTimersByTime(500));
    expect(mocks.loading).toHaveBeenLastCalledWith(
      "Downloading Apple Speech",
      expect.objectContaining({
        id: "downloading-model",
        description: "Starting",
        duration: Infinity,
      }),
    );
    mocks.notifications.activeDownloads = [
      { model: "apple-speech", displayName: "Apple Speech", progress: 42 },
    ];
    view.rerender(<ToastNotifications />);
    expect(mocks.loading).toHaveBeenLastCalledWith(
      "Downloading Apple Speech",
      expect.objectContaining({
        id: "downloading-model",
        description: "42%",
        duration: Infinity,
      }),
    );
    mocks.notifications.hasActiveDownload = false;
    mocks.notifications.activeDownloads = [];
    view.rerender(<ToastNotifications />);
    expect(mocks.dismiss).toHaveBeenCalledWith("downloading-model");
  });

  it("uses the latest registry action while a toast remains visible", () => {
    mocks.dismissedToastIds.add("auth-promotion");
    mocks.config.current_llm_provider = null;
    mocks.config.current_llm_model = null;

    const view = render(<ToastNotifications />);

    act(() => vi.advanceTimersByTime(500));

    const options = mocks.message.mock.calls[0][1];

    mocks.currentTab = { type: "settings", state: { tab: "general" } };
    view.rerender(<ToastNotifications />);

    options.action.onClick();

    expect(mocks.updateSettingsTabState).toHaveBeenCalledWith(
      mocks.currentTab,
      { tab: "intelligence" },
    );
    expect(mocks.openNew).not.toHaveBeenCalled();
  });
});
