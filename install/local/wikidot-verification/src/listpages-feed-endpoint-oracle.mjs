import { createHash } from "node:crypto";
import fs from "node:fs/promises";
import http from "node:http";

export const FEED_ENDPOINT_CASES_SCHEMA =
  "wikijump_listpages_compat.feed_endpoint_cases.v1";
export const FEED_ENDPOINT_CAPTURE_SCHEMA =
  "wikijump_listpages_compat.feed_endpoint_capture.v1";
export const ALLOWED_FEED_SITE = "sandbox-for-codex";
export const ALLOWED_FEED_ORIGIN =
  "http://sandbox-for-codex.wikidot.com";

export function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function requiredString(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${label} must be a nonempty string`);
  }
  return value;
}

export function validateFeedEndpointCases(fixture) {
  if (
    fixture?.schema !== FEED_ENDPOINT_CASES_SCHEMA ||
    fixture.site !== ALLOWED_FEED_SITE ||
    !Array.isArray(fixture.cases) ||
    fixture.cases.length === 0
  ) {
    throw new Error("unsupported ListPages feed endpoint fixture");
  }

  const caseIds = new Set();
  for (const entry of fixture.cases) {
    const caseId = requiredString(entry?.case_id, "case_id");
    const path = requiredString(entry?.path, `${caseId} path`);
    if (
      caseIds.has(caseId) ||
      !path.startsWith("/feed/pages") ||
      path.includes("://") ||
      /[\r\n]/u.test(path)
    ) {
      throw new Error(`unsafe or duplicate feed endpoint case ${caseId}`);
    }
    if (
      !Number.isInteger(entry.expected_status) ||
      !["rss", "error", "html"].includes(entry.expected_kind)
    ) {
      throw new Error(`invalid expectation for feed endpoint case ${caseId}`);
    }
    caseIds.add(caseId);
  }
  for (const entry of fixture.cases) {
    if (
      entry.same_item_guids_as !== undefined &&
      !caseIds.has(entry.same_item_guids_as)
    ) {
      throw new Error(
        `${entry.case_id} names a missing feed comparison case`,
      );
    }
  }
  return fixture;
}

function decodeXmlText(value) {
  return value
    .replaceAll("&#039;", "'")
    .replaceAll("&quot;", '"')
    .replaceAll("&gt;", ">")
    .replaceAll("&lt;", "<")
    .replaceAll("&amp;", "&");
}

function firstElement(body, name) {
  const match = body.match(
    new RegExp(`<${name}>([\\s\\S]*?)</${name}>`, "u"),
  );
  return match ? decodeXmlText(match[1]) : null;
}

export function summarizeFeedEndpointResponse(body, contentType) {
  if (body.startsWith("<?xml") && body.includes("<rss ")) {
    return {
      kind: "rss",
      title: firstElement(body, "title"),
      link: firstElement(body, "link"),
      description: firstElement(body, "description"),
      last_build_date: firstElement(body, "lastBuildDate"),
      item_count: [...body.matchAll(/<item>/gu)].length,
      item_guids: [...body.matchAll(/<guid>([\s\S]*?)<\/guid>/gu)].map(
        (match) => decodeXmlText(match[1]),
      ),
      has_content_encoded_namespace: body.includes(
        'xmlns:content="http://purl.org/rss/1.0/modules/content/"',
      ),
      has_wikidot_namespace: body.includes(
        'xmlns:wikidot="http://www.wikidot.com/rss-namespace"',
      ),
    };
  }
  const processError = body.match(
    /ProcessException' with message '([^']+)'/u,
  );
  if (processError) {
    return {
      kind: "error",
      error: processError[1],
      leaked_server_stack: body.includes("Stack trace:"),
    };
  }
  return {
    kind: contentType.includes("html") ? "html" : "other",
  };
}

export function verifyFeedEndpointCaptures(fixture, captures) {
  const byCaseId = new Map(captures.map((capture) => [capture.case_id, capture]));
  const failures = [];
  for (const entry of fixture.cases) {
    const capture = byCaseId.get(entry.case_id);
    if (!capture) {
      failures.push(`${entry.case_id}: missing capture`);
      continue;
    }
    const summary = capture.summary;
    for (const [field, expected] of [
      ["status", entry.expected_status],
      ["kind", entry.expected_kind],
      ["item_count", entry.expected_item_count],
      ["title", entry.expected_title],
      ["description", entry.expected_description],
      ["link", entry.expected_link],
      ["error", entry.expected_error],
    ]) {
      if (expected !== undefined) {
        const actual = field === "status" ? capture.status : summary[field];
        if (actual !== expected) {
          failures.push(
            `${entry.case_id}: ${field} expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
          );
        }
      }
    }
    if (entry.same_item_guids_as) {
      const other = byCaseId.get(entry.same_item_guids_as);
      if (
        !other ||
        JSON.stringify(summary.item_guids) !==
          JSON.stringify(other.summary.item_guids)
      ) {
        failures.push(
          `${entry.case_id}: item GUIDs differ from ${entry.same_item_guids_as}`,
        );
      }
    }
  }
  return failures;
}

export async function captureFeedEndpointCases(
  fixture,
  {
    requestImpl = anonymousFeedEndpointRequest,
    capturedAt = new Date().toISOString(),
  } = {},
) {
  validateFeedEndpointCases(fixture);
  const captures = [];
  for (const entry of fixture.cases) {
    const requestedUrl = `${ALLOWED_FEED_ORIGIN}${entry.path}`;
    const response = await requestImpl(entry.path, {
      headers: {
        accept: "application/rss+xml, application/xml, text/xml, text/html",
        "user-agent": "Wikijump-ListPages-Compatibility-Oracle/1.0",
      },
    });
    const body = await response.text();
    const contentType = response.headers.get("content-type") ?? "";
    captures.push({
      schema: FEED_ENDPOINT_CAPTURE_SCHEMA,
      case_id: entry.case_id,
      requested_url: requestedUrl,
      final_url: response.url || requestedUrl,
      captured_at: capturedAt,
      provenance: {
        site: ALLOWED_FEED_SITE,
        site_domain: "sandbox-for-codex.wikidot.com",
        method: "anonymous-http-get",
        authenticated: false,
        mutated: false,
      },
      status: response.status,
      headers: {
        cache_control: response.headers.get("cache-control"),
        content_type: contentType,
        expires: response.headers.get("expires"),
        x_frame_options: response.headers.get("x-frame-options"),
      },
      body,
      body_sha256: sha256(body),
      summary: summarizeFeedEndpointResponse(body, contentType),
    });
  }
  return captures;
}

export function anonymousFeedEndpointRequest(path, { headers } = {}) {
  return new Promise((resolve, reject) => {
    const request = http.request(
      {
        hostname: "sandbox-for-codex.wikidot.com",
        method: "GET",
        path,
        headers,
      },
      (response) => {
        response.setEncoding("utf8");
        let body = "";
        response.on("data", (chunk) => {
          body += chunk;
        });
        response.on("end", () => {
          resolve({
            status: response.statusCode ?? 0,
            url: `${ALLOWED_FEED_ORIGIN}${path}`,
            headers: new Headers(
              Object.entries(response.headers).flatMap(([name, value]) =>
                Array.isArray(value)
                  ? value.map((item) => [name, item])
                  : value === undefined
                    ? []
                    : [[name, value]],
              ),
            ),
            text: async () => body,
          });
        });
      },
    );
    request.setTimeout(20_000, () => {
      request.destroy(new Error("ListPages feed endpoint request timed out"));
    });
    request.on("error", reject);
    request.end();
  });
}

export async function readFeedEndpointCases(path) {
  return validateFeedEndpointCases(
    JSON.parse(await fs.readFile(path, "utf8")),
  );
}

export async function writeFeedEndpointCaptures(path, captures) {
  await fs.writeFile(
    path,
    captures.map((capture) => `${JSON.stringify(capture)}\n`).join(""),
  );
}
