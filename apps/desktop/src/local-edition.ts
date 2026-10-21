export const LOCAL_CAPABILITIES = Object.freeze({
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

const VENDOR_HOSTS = new Set([
  "anarlog.so",
  "hyprnote.com",
  "char.com",
  "char.ai",
  "fastrepl.com",
]);

export function isVendorServiceUrl(value: string | URL): boolean {
  let url: URL;
  try {
    url = value instanceof URL ? value : new URL(value);
  } catch {
    return false;
  }

  const host = url.hostname.toLowerCase().replace(/\.$/u, "");
  return [...VENDOR_HOSTS].some(
    (vendorHost) => host === vendorHost || host.endsWith(`.${vendorHost}`),
  );
}

export function assertAllowedProviderUrl(value: string): string {
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    throw new Error("Provider endpoint must be a valid URL");
  }

  if (!["http:", "https:"].includes(url.protocol)) {
    throw new Error("Provider endpoint must use HTTP or HTTPS");
  }
  if (isVendorServiceUrl(url)) {
    throw new Error(`${url.hostname} is disabled in Anarlog Local`);
  }
  return value;
}

export function installVendorNetworkGuard(
  target: Pick<typeof globalThis, "fetch"> = globalThis,
): () => void {
  const originalFetch = target.fetch.bind(target);
  target.fetch = (async (input: RequestInfo | URL, init?: RequestInit) => {
    const url =
      typeof input === "string"
        ? input
        : input instanceof URL
          ? input.toString()
          : input.url;
    assertAllowedProviderUrl(url);
    return originalFetch(input, init);
  }) as typeof fetch;

  return () => {
    target.fetch = originalFetch;
  };
}
