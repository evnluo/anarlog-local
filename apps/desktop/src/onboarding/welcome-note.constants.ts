export const WELCOME_NOTE_DEMO_URL = "";
export const WELCOME_NOTE_TRACKING_ID = "anarlog-onboarding-demo-v1";
const WELCOME_NOTE_COMPLETE_PATH = "/onboarding-demo/complete";
const WELCOME_NOTE_DEMO_AUTOJOIN_PARAM = "autojoin";

export function buildWelcomeNoteDemoUrl(meetingLink: string, port?: number) {
  if (!meetingLink) return "";
  const url = new URL(meetingLink);
  url.searchParams.set(WELCOME_NOTE_DEMO_AUTOJOIN_PARAM, "1");
  if (port != null) {
    url.searchParams.set(
      "completion_url",
      `http://127.0.0.1:${port}${WELCOME_NOTE_COMPLETE_PATH}`,
    );
  }
  return url.toString();
}
