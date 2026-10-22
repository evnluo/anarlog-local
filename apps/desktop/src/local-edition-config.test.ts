import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const desktopRoot = resolve(import.meta.dirname, "..");
const source = (relativePath: string) =>
  readFileSync(resolve(desktopRoot, relativePath), "utf8");

describe("Anarlog Local distribution identity", () => {
  const config = JSON.parse(source("src-tauri/tauri.conf.json")) as {
    productName: string;
    mainBinaryName: string;
    identifier: string;
    version: string;
    bundle: { publisher: string; createUpdaterArtifacts: boolean };
    plugins?: { updater?: { active?: boolean } };
  };

  it("uses a separate name, bundle identifier, and binary", () => {
    expect(config.productName).toBe("Anarlog Local");
    expect(config.mainBinaryName).toBe("anarlog-local");
    expect(config.identifier).toBe("com.evnluo.anarlog-local");
    expect(config.version).toBe("0.1.0");
    expect(config.bundle.publisher).not.toBe("Fastrepl");
  });

  it("does not build or activate vendor updater artifacts", () => {
    expect(config.bundle.createUpdaterArtifacts).toBe(false);
    expect(config.plugins?.updater?.active).toBe(false);
    const nativeSource = source("src-tauri/src/lib.rs");
    expect(nativeSource).not.toContain("plugin(tauri_plugin_updater");
    expect(nativeSource).toContain("plugin(tauri_plugin_tray::init(false))");
    expect(nativeSource).toContain("plugin(tauri_plugin_db::init(db.clone()))");
    expect(nativeSource).not.toContain("init_with_cloudsync");
    expect(source("src/settings/queries.ts")).not.toContain(
      "@anlg/plugin-updater2",
    );
    expect(source("src/services/event-listeners.tsx")).not.toContain(
      "@anlg/plugin-updater2",
    );
    expect(source("src/sidebar/toast/index.tsx")).not.toContain(
      "~/main/update-banner",
    );
    expect(source("src/sidebar/toast/registry.tsx")).not.toContain(
      "Upgrade to Pro",
    );
    expect(source("src/folders/folder-editor.tsx")).not.toContain(
      "ResourceShareButton",
    );
    expect(source("src/templates/template-form.tsx")).not.toContain(
      "ResourceShareButton",
    );
  });

  it("has no vendor static-asset default", () => {
    expect(source("src/env.ts")).not.toMatch(/static\.anarlog\.so/u);
  });
});
