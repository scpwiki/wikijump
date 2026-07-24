import {execFile} from "node:child_process";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {promisify} from "node:util";

export const browserRenderExecFile = promisify(execFile);
const supportDirectory = path.dirname(fileURLToPath(import.meta.url));
export const browserRenderTestDirectory = path.resolve(supportDirectory, "..");
export const browserRenderScriptPath = path.resolve(
  browserRenderTestDirectory,
  "../scripts/capture-browser-rendering.mjs",
);

export const browserRenderInventory = {
  schema: "wikijump_full_parity.corpus_inventory_lock.v1",
  rows: [
    {
      fixture_id: "EN:alpha",
      family: "EN",
      slug: "alpha",
      source_url: "https://scp-wiki.wikidot.com/alpha",
      local_https_url: "https://scp-wiki.wikijump.localhost/alpha",
      required_browser: true,
    },
    {
      fixture_id: "EN:beta",
      family: "EN",
      slug: "beta",
      source_url: "https://scp-wiki.wikidot.com/beta",
      local_https_url: "https://scp-wiki.wikijump.localhost/beta",
      required_browser: true,
    },
  ],
};
