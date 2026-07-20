import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { capturePng } from "../src/standing-browser-screenshot.mjs";

test("CDP screenshot capture preserves immediate viewport and settled full-page modes", async (context) => {
  const directory = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-screenshot-"),
  );
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const calls = [];
  const client = {
    async send(method, options) {
      calls.push({ method, options });
      if (method === "Page.getLayoutMetrics") {
        return { cssContentSize: { width: 100, height: 200 } };
      }
      return { data: Buffer.from("png").toString("base64") };
    },
    async detach() {
      calls.push({ method: "detach" });
    },
  };
  const page = { context: () => ({ newCDPSession: async () => client }) };
  const viewport = path.join(directory, "viewport.png");
  const full = path.join(directory, "full.png");
  await capturePng(page, viewport);
  await capturePng(page, full, { fullPage: true });
  assert.equal((await fs.readFile(viewport)).toString(), "png");
  assert.equal((await fs.readFile(full)).toString(), "png");
  assert.deepEqual(
    calls
      .filter((call) => call.method === "Page.captureScreenshot")
      .map((call) => call.options.captureBeyondViewport),
    [false, true],
  );
});
