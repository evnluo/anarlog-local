import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  platform: vi.fn(() => "macos"),
}));

vi.mock("@tauri-apps/plugin-os", () => ({
  platform: mocks.platform,
}));

import { AppSettingsView } from "./app-settings";

function setting(value = true) {
  return {
    value,
    onChange: vi.fn(),
  };
}

function renderAppSettings({ appStoreBuild = false } = {}) {
  return render(
    <AppSettingsView
      appStoreBuild={appStoreBuild}
      autostart={setting()}
      showAppInDock={setting()}
      showTrayIcon={setting()}
    />,
  );
}

describe("AppSettingsView", () => {
  afterEach(() => {
    cleanup();
    mocks.platform.mockReturnValue("macos");
  });

  it("does not offer upstream release notes", () => {
    renderAppSettings();

    expect(
      screen.queryByRole("button", { name: "Show release notes" }),
    ).toBeNull();
  });

  it("does not offer an updater control without an updater feed", () => {
    renderAppSettings();

    expect(
      screen.queryByRole("switch", { name: "Automatically install updates" }),
    ).toBeNull();
  });

  it("hides direct-distribution controls in App Store builds", () => {
    renderAppSettings({ appStoreBuild: true });

    expect(
      screen.queryByRole("switch", { name: "Start Anarlog Local at login" }),
    ).toBeNull();
    expect(
      screen.queryByRole("switch", { name: "Automatically install updates" }),
    ).toBeNull();
  });
});
