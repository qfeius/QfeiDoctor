import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

import {
  createReleaseConfig,
  parseReleaseVersion,
} from "./prepare-release-config.mjs";

describe("release version", () => {
  it("uses the semantic version from a release tag", () => {
    expect(parseReleaseVersion("v1.2.3")).toBe("1.2.3");
  });

  it.each(["1.2.3", "v1.2", "v01.2.3", "release-v1.2.3"])(
    "rejects invalid release tag %s",
    (tag) => {
      expect(() => parseReleaseVersion(tag)).toThrow(
        `Invalid release tag: ${tag}`,
      );
    },
  );

  it("creates a Tauri config override", () => {
    expect(createReleaseConfig("v1.2.3")).toEqual({ version: "1.2.3" });
  });

  it("injects the tag-derived config into release builds", () => {
    const workflow = readFileSync(".github/workflows/release.yml", "utf8");

    expect(workflow).toContain("RELEASE_TAG: ${{ github.ref_name }}");
    expect(workflow).toContain("run: node scripts/prepare-release-config.mjs");
    expect(workflow).toContain(
      "args: --config src-tauri/tauri.release.conf.json",
    );
  });
});
