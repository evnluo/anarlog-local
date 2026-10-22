import type { JsonValue } from "@anlg/plugin-analytics";

export function trackAnalyticsEvent(
  _event: string,
  _properties: Record<string, JsonValue> = {},
) {
  // Anarlog Local never sends product analytics.
}
