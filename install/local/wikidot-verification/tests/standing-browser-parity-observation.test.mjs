import assert from "node:assert/strict";
import test from "node:test";

import { observationArtifactName } from "../src/standing-browser-parity-observation.mjs";

test("immediate and settled browser artifacts have deterministic, distinct safe names", () => {
  const input = {
    label: "local",
    index: 0,
    url: "https://scp-wiki.wikijump.localhost:18443/scp-9506",
  };
  const immediate = observationArtifactName({
    ...input,
    phase: "domcontentloaded-immediate",
  });
  const viewport = observationArtifactName({
    ...input,
    phase: "settled-viewport",
  });
  const fullPage = observationArtifactName({
    ...input,
    phase: "settled-full-page",
  });
  assert.match(
    immediate,
    /^standing-browser-local-00-[0-9a-f]{16}-domcontentloaded-immediate\.png$/u,
  );
  assert.notEqual(immediate, viewport);
  assert.notEqual(viewport, fullPage);
  assert.throws(
    () => observationArtifactName({ ...input, phase: "first-paint" }),
    /unsupported browser observation artifact phase/u,
  );
});
