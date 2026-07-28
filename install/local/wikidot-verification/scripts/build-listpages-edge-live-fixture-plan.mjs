#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";

import {
  buildListPagesEdgeLiveFixturePlan,
} from "../src/listpages-live-fixture-plan.mjs";

const outputIndex = process.argv.indexOf("--output");
if (outputIndex === -1 || !process.argv[outputIndex + 1]) {
  throw new Error("--output is required");
}

const output = path.resolve(process.argv[outputIndex + 1]);
const plan = buildListPagesEdgeLiveFixturePlan();
await fs.mkdir(path.dirname(output), { recursive: true });
await fs.writeFile(output, `${JSON.stringify(plan, null, 2)}\n`, {
  encoding: "utf8",
  flag: "wx",
});
console.log(JSON.stringify({
  output,
  pages: plan.pages.length,
  captures: plan.captures.length,
}));
