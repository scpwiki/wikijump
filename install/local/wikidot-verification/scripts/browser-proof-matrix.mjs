#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const verifierRoot = path.resolve(__dirname, "..");
const corpusRoot = path.join(verifierRoot, "corpus");
const manifestPath = path.join(corpusRoot, "manifest.json");
const repoRoot = path.resolve(verifierRoot, "../../..");
const chromium = loadPlaywrightChromium();

function loadPlaywrightChromium() {
  const candidates = [
    { base: path.join(repoRoot, "framerail", "package.json"), names: ["@playwright/test", "playwright"] },
    { base: "/home/roku/.npm-global/lib/node_modules/playwright/package.json", names: ["playwright"] },
    { base: path.join(process.env.HOME || "/home/roku", ".npm-global/lib/node_modules/playwright/package.json"), names: ["playwright"] },
  ];

  const failures = [];
  for (const candidate of candidates) {
    try {
      const candidateRequire = createRequire(candidate.base);
      for (const name of candidate.names) {
        try {
          const mod = candidateRequire(name);
          if (mod.chromium) return mod.chromium;
        } catch (error) {
          failures.push(`${candidate.base} -> ${name}: ${error.code || error.message}`);
        }
      }
    } catch (error) {
      failures.push(`${candidate.base}: ${error.code || error.message}`);
    }
  }

  throw new Error(`Unable to load Playwright chromium. Tried:\n${failures.join("\n")}`);
}

function parseArgs(argv) {
  const args = {
    baseUrl: process.env.WIKIDOT_VERIFY_BASE_URL || "https://scpwiki.localhost",
    outputDir: path.resolve(process.cwd(), "wikidot-browser-proof"),
  };

  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--base-url") {
      args.baseUrl = argv[++index].replace(/\/$/, "");
    } else if (arg === "--output-dir") {
      args.outputDir = path.resolve(argv[++index]);
    } else if (arg === "--headed") {
      args.headed = true;
    } else if (arg === "--help") {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return args;
}

function printHelpAndExit() {
  console.log("Usage: node browser-proof-matrix.mjs [--base-url URL] [--output-dir DIR] [--headed]");
  process.exit(0);
}

async function readJson(file) {
  return JSON.parse(await fs.readFile(file, "utf8"));
}

function proofEntries(manifest) {
  const pages = manifest.pages
    .filter((page) => page.proof)
    .map((page) => ({ ...page, proof: page.proof }));
  return [...pages, {
    slug: manifest.editProof.slug,
    title: manifest.editProof.title,
    proof: manifest.editProof.proof,
  }];
}

function isLocalUrl(url) {
  const parsed = new URL(url);
  return parsed.hostname === "127.0.0.1" ||
    parsed.hostname === "localhost" ||
    parsed.hostname.endsWith(".localhost") ||
    parsed.hostname === "0.0.0.0";
}

function isAllowedExternalUrl(url, proof) {
  if (isLocalUrl(url)) return true;

  const parsed = new URL(url);
  const allowedHosts = new Set([
    "d3g0gp89917ko0.cloudfront.net",
    "cdn.scpwiki.com",
    "scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com",
    "rsms.me",
    "maxcdn.bootstrapcdn.com",
    "scp-wiki.wdfiles.com",
    ...(proof.allowedExternalHosts || []),
  ]);

  return allowedHosts.has(parsed.hostname);
}

async function evaluateImages(page) {
  return page.evaluate(() => {
    const images = Array.from(document.images);
    return {
      imageCount: images.length,
      loadedImages: images.filter((image) => image.complete && image.naturalWidth > 0 && image.naturalHeight > 0).length,
      sources: images.map((image) => ({
        src: image.currentSrc || image.src,
        complete: image.complete,
        naturalWidth: image.naturalWidth,
        naturalHeight: image.naturalHeight,
      })),
    };
  });
}

async function runProofForPage(browser, entry, args, dirs) {
  const pageUrl = `${args.baseUrl}/${entry.slug}`;
  const page = await browser.newPage({
    viewport: { width: 1366, height: 900 },
  });
  const network = {
    requests: [],
    failedRequests: [],
    badResponses: [],
    externalRequests: [],
  };

  page.on("request", (request) => {
    const url = request.url();
    network.requests.push({ method: request.method(), url, resourceType: request.resourceType() });
    if (entry.proof.requireLocalNetworkOnly && !isAllowedExternalUrl(url, entry.proof)) {
      network.externalRequests.push({ method: request.method(), url, resourceType: request.resourceType() });
    }
  });
  page.on("requestfailed", (request) => {
    network.failedRequests.push({
      method: request.method(),
      url: request.url(),
      resourceType: request.resourceType(),
      failure: request.failure()?.errorText || "unknown",
    });
  });
  page.on("response", (response) => {
    const status = response.status();
    if (status >= 400) {
      network.badResponses.push({ status, url: response.url() });
    }
  });

  const checks = [];
  let ok = true;
  const check = (name, pass, detail = "") => {
    checks.push({ name, pass, detail });
    if (!pass) ok = false;
  };

  try {
    const response = await page.goto(pageUrl, { waitUntil: "networkidle", timeout: 45000 });
    check("http-status", Boolean(response && response.ok()), response ? String(response.status()) : "no response");

    const bodyText = await page.locator("body").innerText({ timeout: 10000 });
    for (const text of entry.proof.requiredText || []) {
      check(`required-text:${text}`, bodyText.includes(text));
    }
    for (const text of entry.proof.absentText || []) {
      check(`absent-text:${text}`, !bodyText.includes(text));
    }

    for (const selector of entry.proof.requireSelectors || []) {
      check(`selector:${selector}`, await page.locator(selector).count() > 0);
    }
    for (const selectorText of entry.proof.selectorText || []) {
      const text = await page.locator(selectorText.selector).first().innerText({ timeout: 5000 }).catch(() => "");
      check(`selector-text:${selectorText.selector}`, text.includes(selectorText.text), text);
    }

    if (entry.proof.cssColor) {
      const color = await page.locator(entry.proof.cssColor.selector).first().evaluate((element, property) => {
        return getComputedStyle(element).getPropertyValue(property);
      }, entry.proof.cssColor.property).catch((error) => `ERROR:${error.message}`);
      check(`css:${entry.proof.cssColor.selector}:${entry.proof.cssColor.property}`, color.trim() === entry.proof.cssColor.expected, color.trim());
    }

    const images = await evaluateImages(page);
    if (entry.proof.minImages !== undefined) {
      check("image-count", images.imageCount >= entry.proof.minImages, String(images.imageCount));
    }
    if (entry.proof.expectedLoadedImages !== undefined) {
      check("loaded-images", images.loadedImages >= entry.proof.expectedLoadedImages, `${images.loadedImages}/${images.imageCount}`);
    }

    check("network-failed-requests", network.failedRequests.length === 0, String(network.failedRequests.length));
    check("network-bad-responses", network.badResponses.length === 0, String(network.badResponses.length));
    if (entry.proof.requireLocalNetworkOnly) {
      check("network-local-only", network.externalRequests.length === 0, String(network.externalRequests.length));
    }

    await page.screenshot({ path: path.join(dirs.screenshots, `${entry.slug.replaceAll(":", "_")}.png`), fullPage: true });
    await fs.writeFile(path.join(dirs.network, `${entry.slug.replaceAll(":", "_")}.json`), JSON.stringify({
      slug: entry.slug,
      url: pageUrl,
      checks,
      images,
      network,
    }, null, 2));

    return {
      slug: entry.slug,
      url: pageUrl,
      ok,
      checks,
      imageCount: images.imageCount,
      loadedImages: images.loadedImages,
      failedRequestCount: network.failedRequests.length,
      badResponseCount: network.badResponses.length,
      externalRequestCount: network.externalRequests.length,
    };
  } catch (error) {
    await page.screenshot({ path: path.join(dirs.screenshots, `${entry.slug.replaceAll(":", "_")}-error.png`), fullPage: true }).catch(() => {});
    return {
      slug: entry.slug,
      url: pageUrl,
      ok: false,
      checks: [{ name: "exception", pass: false, detail: error.stack || error.message }],
      imageCount: 0,
      loadedImages: 0,
      failedRequestCount: network.failedRequests.length,
      badResponseCount: network.badResponses.length,
      externalRequestCount: network.externalRequests.length,
    };
  } finally {
    await page.close().catch(() => {});
  }
}

async function main() {
  const args = parseArgs(process.argv);
  const manifest = await readJson(manifestPath);
  const dirs = {
    root: args.outputDir,
    screenshots: path.join(args.outputDir, "screenshots"),
    network: path.join(args.outputDir, "network"),
  };
  await fs.mkdir(dirs.screenshots, { recursive: true });
  await fs.mkdir(dirs.network, { recursive: true });

  const entries = proofEntries(manifest);
  const browser = await chromium.launch({ headless: !args.headed });
  const results = [];
  try {
    for (const entry of entries) {
      results.push(await runProofForPage(browser, entry, args, dirs));
    }
  } finally {
    await browser.close();
  }

  const summary = {
    generatedAt: new Date().toISOString(),
    baseUrl: args.baseUrl,
    verifierRoot,
    pageCount: results.length,
    passed: results.filter((result) => result.ok).length,
    failed: results.filter((result) => !result.ok).length,
    results,
  };
  const summaryPath = path.join(args.outputDir, "browser-summary.json");
  await fs.writeFile(summaryPath, JSON.stringify(summary, null, 2));
  await fs.writeFile(path.join(args.outputDir, "fixture-results.tsv"), [
    "slug\tok\timages\tfailed_requests\tbad_responses\texternal_requests\tfailed_checks",
    ...results.map((result) => [
      result.slug,
      result.ok ? "PASS" : "FAIL",
      `${result.loadedImages}/${result.imageCount}`,
      result.failedRequestCount,
      result.badResponseCount,
      result.externalRequestCount,
      result.checks.filter((check) => !check.pass).map((check) => `${check.name}:${check.detail}`).join(" | "),
    ].join("\t")),
    "",
  ].join("\n"));

  console.log(`Browser proof: ${summary.passed}/${summary.pageCount} passed.`);
  console.log(`Summary: ${summaryPath}`);
  if (summary.failed > 0) {
    process.exit(1);
  }
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
