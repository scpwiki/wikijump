#!/usr/bin/env node

import fs from "node:fs";
import https from "node:https";

import {runCliIfMain} from "../src/cli-entry.mjs";
import {validateSavedPageRerenderReceipt} from "../src/saved-page-runtime-rerender.mjs";
import {
  compareSavedPageRuntime,
  selectSavedPageReferences,
  validateRuntimeIdentity,
} from "../src/saved-page-runtime-differential.mjs";
import {sha256} from "../src/syntax-differential.mjs";

function valueAfter(argv, index, option) {
  const value = argv[index + 1];
  if (value == null || value.startsWith("--")) throw new Error(`${option} requires a value`);
  return value;
}

function parseArgs(argv) {
  const args = {
    references: null,
    runtimeIdentity: null,
    rerenderReceipt: null,
    caseIds: [],
    localBase: null,
    output: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--references") args.references = valueAfter(argv, index++, option);
    else if (option === "--runtime-identity") args.runtimeIdentity = valueAfter(argv, index++, option);
    else if (option === "--rerender-receipt") args.rerenderReceipt = valueAfter(argv, index++, option);
    else if (option === "--case-id") args.caseIds.push(valueAfter(argv, index++, option));
    else if (option === "--local-base") args.localBase = valueAfter(argv, index++, option);
    else if (option === "--output") args.output = valueAfter(argv, index++, option);
    else throw new Error(`unknown option: ${option}`);
  }
  for (const [name, value] of Object.entries(args)) {
    if (!value) throw new Error(`--${name.replace(/[A-Z]/gu, (letter) => `-${letter.toLowerCase()}`)} is required`);
  }
  const base = new URL(args.localBase);
  if (base.protocol !== "https:" || !base.hostname.endsWith(".wikijump.localhost")) {
    throw new Error("--local-base must be one local Wikijump HTTPS origin");
  }
  args.localBase = base;
  return args;
}

async function fetchLocal(reference, localBase) {
  const url = new URL(`/${reference.page.slug}`, localBase);
  const html = await new Promise((resolve, reject) => {
    const request = https.get(
      url,
      (response) => {
        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`local runtime returned ${response.statusCode} for ${url}`));
          return;
        }
        const chunks = [];
        response.on("data", (chunk) => chunks.push(chunk));
        response.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
      },
    );
    request.on("error", reject);
  });
  return {url: url.href, html};
}

export async function main(argv) {
  const args = parseArgs(argv);
  const references = selectSavedPageReferences(
    fs
      .readFileSync(args.references, "utf8")
      .split("\n")
      .filter((line) => line.trim())
      .map((line) => JSON.parse(line)),
    args.caseIds,
  );
  const runtimeIdentity = validateRuntimeIdentity(
    JSON.parse(fs.readFileSync(args.runtimeIdentity, "utf8")),
  );
  const rerenderReceiptContents = fs.readFileSync(args.rerenderReceipt, "utf8");
  const rerenderReceipt = validateSavedPageRerenderReceipt(
    JSON.parse(rerenderReceiptContents),
    references,
    runtimeIdentity,
  );
  const comparisons = [];
  for (const reference of references) {
    const local = await fetchLocal(reference, args.localBase);
    comparisons.push({
      ...compareSavedPageRuntime(reference, local.html, runtimeIdentity),
      local_url: local.url,
    });
  }
  const report = {
    schema: "wikijump_syntax_differential.saved_page_runtime_verdict.v1",
    runtime_identity: runtimeIdentity,
    rerender_receipt: {
      schema: rerenderReceipt.schema,
      sha256: sha256(rerenderReceiptContents),
    },
    summary: {
      total: comparisons.length,
      match: comparisons.filter((comparison) => comparison.status === "match").length,
      mismatch: comparisons.filter((comparison) => comparison.status === "mismatch").length,
    },
    comparisons,
  };
  fs.writeFileSync(args.output, `${JSON.stringify(report, null, 2)}\n`, {flag: "wx"});
  console.log(JSON.stringify(report.summary));
  return report.summary.mismatch === 0 ? 0 : 1;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
