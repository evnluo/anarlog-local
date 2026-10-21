import { describe, expect, it, vi } from "vitest";

import {
  LOCAL_CAPABILITIES,
  assertAllowedProviderUrl,
  installVendorNetworkGuard,
  isVendorServiceUrl,
} from "./local-edition";

describe("Anarlog Local capabilities", () => {
  it("enables local features without enabling hosted services", () => {
    expect(LOCAL_CAPABILITIES).toMatchObject({
      playbackSpeed: true,
      dictionary: true,
      dictation: true,
      customTemplates: true,
      localAutomations: true,
      managedAi: false,
      cloudSync: false,
      sharing: false,
      teams: false,
      billing: false,
      vendorTelemetry: false,
      vendorUpdates: false,
    });
  });
});

describe("vendor network boundary", () => {
  it.each([
    "https://anarlog.so",
    "https://api.anarlog.so/v1",
    "https://static.anarlog.so/model",
    "https://hyprnote.com",
    "https://api.hyprnote.com",
    "https://api.char.com/v1",
    "https://char.ai/oauth",
    "https://fastrepl.com",
  ])("blocks %s", (url) => {
    expect(isVendorServiceUrl(url)).toBe(true);
    expect(() => assertAllowedProviderUrl(url)).toThrow(
      /disabled in Anarlog Local/u,
    );
  });

  it.each([
    "http://127.0.0.1:11434/v1",
    "http://localhost:1234/v1",
    "https://api.openai.com/v1",
    "https://api.anthropic.com/v1",
    "https://openrouter.ai/api/v1",
    "https://huggingface.co/model",
  ])("allows explicit local, BYOK, or model URLs: %s", (url) => {
    expect(isVendorServiceUrl(url)).toBe(false);
    expect(assertAllowedProviderUrl(url)).toBe(url);
  });

  it("rejects malformed provider URLs", () => {
    expect(() => assertAllowedProviderUrl("not a URL")).toThrow(/valid URL/u);
  });

  it("blocks vendor fetches before the network implementation runs", async () => {
    const baseFetch = vi.fn(async () => new Response("ok"));
    const target = { fetch: baseFetch as typeof fetch };
    const restore = installVendorNetworkGuard(target);

    await expect(target.fetch("https://api.anarlog.so/v1")).rejects.toThrow(
      /disabled in Anarlog Local/u,
    );
    expect(baseFetch).not.toHaveBeenCalled();

    await expect(
      target.fetch("https://api.openai.com/v1"),
    ).resolves.toBeInstanceOf(Response);
    expect(baseFetch).toHaveBeenCalledOnce();

    restore();
    expect(target.fetch).not.toBe(baseFetch);
    await target.fetch("https://api.anarlog.so/v1");
    expect(baseFetch).toHaveBeenCalledTimes(2);
  });
});
