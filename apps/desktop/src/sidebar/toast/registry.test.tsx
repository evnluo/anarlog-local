import { describe, expect, it } from "vitest";

import { createToastRegistry, getToastToShow } from "./registry";

type RegistryParams = Parameters<typeof createToastRegistry>[0];

const baseParams: RegistryParams = {
  hasLLMConfigured: true,
  hasSttConfigured: true,
  isAiTranscriptionTabActive: false,
  isAiIntelligenceTabActive: false,
  isBatchTranscribingInActiveTranscriptTab: false,
  hasActiveDownload: false,
  downloadingModel: null,
  activeDownloads: [],
  localSttStatus: null,
  isLocalSttModel: false,
  onOpenLLMSettings: () => undefined,
  onOpenSTTSettings: () => undefined,
};

type Toast = NonNullable<ReturnType<typeof getToastToShow>>;

function showToast(
  overrides: Partial<RegistryParams> = {},
  isDismissed: (toast: Toast) => boolean = () => false,
) {
  return getToastToShow(
    createToastRegistry({ ...baseParams, ...overrides }),
    isDismissed,
  );
}

describe("sidebar toast registry", () => {
  it.each([
    ["missing-llm", { hasLLMConfigured: false }],
    ["missing-stt", { hasSttConfigured: false }],
  ] as const)(
    "requires %s setup even if it was dismissed previously",
    (id, overrides) => {
      const toast = showToast(overrides, (toast) => toast.id === id);

      expect(toast?.id).toBe(id);
      expect(toast?.lifecycle).toEqual({ type: "condition-bound" });
    },
  );

  it("shows local STT loading unless the transcript tab shows batch progress", () => {
    const loading = {
      localSttStatus: "loading",
      isLocalSttModel: true,
    } as const;

    expect(showToast(loading)?.id).toBe("local-stt-loading");
    expect(
      showToast({ ...loading, isBatchTranscribingInActiveTranscriptTab: true }),
    ).toBeNull();
  });

  it("lets users dismiss a model download toast for the current download", () => {
    const toast = showToast({
      hasActiveDownload: true,
      downloadingModel: "apple-speech",
      activeDownloads: [
        { model: "apple-speech", displayName: "apple-speech", progress: 0 },
      ],
    });

    expect(toast).toMatchObject({
      id: "downloading-model",
      lifecycle: { type: "persistent", dismissal: "session" },
      loading: true,
    });
  });
});
