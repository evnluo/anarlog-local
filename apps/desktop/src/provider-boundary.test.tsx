import { describe, expect, it } from "vitest";

import { PROVIDERS as LLM_PROVIDERS } from "~/settings/ai/llm/shared";
import { PROVIDERS as STT_PROVIDERS } from "~/settings/ai/stt/shared";

describe("Anarlog Local provider catalogue", () => {
  it("does not expose the managed Anarlog LLM", () => {
    expect(LLM_PROVIDERS.map((provider) => provider.id)).not.toContain(
      "anarlog",
    );
    expect(LLM_PROVIDERS.map((provider) => provider.id)).toEqual(
      expect.arrayContaining(["ollama", "openai", "anthropic", "openrouter"]),
    );
  });

  it("does not expose managed Anarlog transcription", () => {
    expect(STT_PROVIDERS.map((provider) => provider.id)).not.toContain(
      "anarlog",
    );
    expect(STT_PROVIDERS.map((provider) => provider.id)).toEqual(
      expect.arrayContaining([
        "soniqo",
        "apple_speech",
        "local_file",
        "deepgram",
      ]),
    );
  });
});
