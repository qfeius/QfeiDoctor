import tauriConfigText from "../../../src-tauri/tauri.conf.json?raw";
import { describe, expect, it } from "vitest";

function loadTauriConfig() {
  return JSON.parse(tauriConfigText) as {
    plugins?: {
      shell?: {
        open?: string | string[] | boolean;
      };
    };
  };
}

function toOpenScopeRegex(): RegExp | null {
  const openScope = loadTauriConfig().plugins?.shell?.open;

  if (typeof openScope === "boolean") {
    return openScope ? /^((mailto:\w+)|(tel:\w+)|(https?:\/\/\w+)).+/ : null;
  }

  if (typeof openScope === "string") {
    return new RegExp(`^${openScope}$`);
  }

  return /^((mailto:\w+)|(tel:\w+)|(https?:\/\/\w+)).+/;
}

describe("tauri shell open scope", () => {
  it("uses a single regex string supported by tauri-plugin-shell", () => {
    expect(typeof loadTauriConfig().plugins?.shell?.open).toBe("string");
  });

  it("allows Windows proxy settings URI", () => {
    expect(toOpenScopeRegex()?.test("ms-settings:network-proxy")).toBe(true);
  });
});
