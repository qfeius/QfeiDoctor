import { writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const RELEASE_TAG_PATTERN = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
const RELEASE_CONFIG_PATH = "src-tauri/tauri.release.conf.json";

export function parseReleaseVersion(tag) {
  const match = RELEASE_TAG_PATTERN.exec(tag);

  if (!match) {
    throw new Error(`Invalid release tag: ${tag}`);
  }

  return `${match[1]}.${match[2]}.${match[3]}`;
}

export function createReleaseConfig(tag) {
  return { version: parseReleaseVersion(tag) };
}

export function prepareReleaseConfig(tag = process.env.RELEASE_TAG) {
  if (!tag) {
    throw new Error("RELEASE_TAG is required");
  }

  console.info(`Preparing Tauri release config for ${tag}`);
  const config = createReleaseConfig(tag);
  writeFileSync(RELEASE_CONFIG_PATH, `${JSON.stringify(config, null, 2)}\n`);
  console.info(`Wrote ${RELEASE_CONFIG_PATH} with version ${config.version}`);
}

const isEntryPoint =
  process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href;

if (isEntryPoint) {
  try {
    prepareReleaseConfig();
  } catch (error) {
    console.error("Failed to prepare Tauri release config", error);
    process.exitCode = 1;
  }
}
