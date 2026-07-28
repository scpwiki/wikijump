#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  captureFeedEndpointCases,
  readFeedEndpointCases,
  verifyFeedEndpointCaptures,
  writeFeedEndpointCaptures,
} from "../src/listpages-feed-endpoint-oracle.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const fixturePath =
  process.argv[2] ??
  path.join(
    root,
    "fixtures",
    "listpages-campaign-feed-endpoint-cases.json",
  );
const outputPath =
  process.argv[3] ??
  path.join(
    root,
    "artifacts",
    "listpages-campaign-feed-endpoint-live.jsonl",
  );

const fixture = await readFeedEndpointCases(fixturePath);
const captures = await captureFeedEndpointCases(fixture);
const failures = verifyFeedEndpointCaptures(fixture, captures);
await writeFeedEndpointCaptures(outputPath, captures);

if (failures.length > 0) {
  throw new Error(
    `ListPages feed endpoint oracle found ${failures.length} discrepancy(s):\n${failures.join("\n")}`,
  );
}

console.log(
  `Captured and verified ${captures.length} live ListPages feed endpoint cases at ${outputPath}`,
);
