import tauriConfigText from "../../../src-tauri/tauri.conf.json?raw";
import { describe, expect, it } from "vitest";

interface TauriConfig {
  bundle?: {
    windows?: {
      webviewInstallMode?: {
        type?: string;
      };
    };
  };
}

function loadTauriConfig(): TauriConfig {
  return JSON.parse(tauriConfigText) as TauriConfig;
}

describe("Tauri Windows installer", () => {
  it("bundles an offline WebView2 installer", () => {
    expect(loadTauriConfig().bundle?.windows?.webviewInstallMode).toEqual({
      type: "offlineInstaller",
    });
  });
});
