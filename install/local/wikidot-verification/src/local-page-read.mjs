import { validateLocalDeepwellRpcUrl } from "./theme-localization-deepwell-adapter.mjs";
import { assertTimestamp } from "./reference-acquisition-inventory-source.mjs";

const MAX_LOCAL_RPC_RESPONSE_BYTES = 32 * 1024 * 1024;
const RFC3339_RE =
  /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})(?:\.(\d+))?(Z|[+-]\d{2}:\d{2})$/u;

export class LocalPageReadError extends Error {
  constructor(code) {
    super(`local page read failed: ${code}`);
    this.code = code;
  }
}

function timestampParts(value) {
  try {
    assertTimestamp(value, "local page_updated_at");
  } catch {
    return null;
  }
  const match = RFC3339_RE.exec(value);
  if (match === null) return null;
  const milliseconds = Date.parse(`${match[1]}${match[3]}`);
  if (!Number.isSafeInteger(milliseconds)) return null;
  return Object.freeze({
    fraction: (match[2] ?? "").replace(/0+$/u, ""),
    seconds: BigInt(Math.trunc(milliseconds / 1000)),
  });
}

export function sameTimestamp(left, right) {
  const leftParts = timestampParts(left);
  const rightParts = timestampParts(right);
  if (
    leftParts === null ||
    rightParts === null ||
    leftParts.seconds !== rightParts.seconds
  ) {
    return false;
  }
  const width = Math.max(leftParts.fraction.length, rightParts.fraction.length);
  return (
    leftParts.fraction.padEnd(width, "0") ===
    rightParts.fraction.padEnd(width, "0")
  );
}

function wellFormedString(value) {
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index);
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) {
        throw new LocalPageReadError("ill_formed_unicode");
      }
      index += 1;
    } else if (code >= 0xdc00 && code <= 0xdfff) {
      throw new LocalPageReadError("ill_formed_unicode");
    }
  }
  return value;
}

function localPage(value, { fullname, siteId }) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new LocalPageReadError("invalid_page_result");
  }
  if (
    value.site_id !== siteId ||
    typeof value.slug !== "string" ||
    value.slug !== fullname
  ) {
    throw new LocalPageReadError("returned_page_identity");
  }
  if (
    typeof value.wikitext !== "string" ||
    typeof value.compiled_body_html !== "string" ||
    !Array.isArray(value.compiled_body_styles) ||
    value.compiled_body_styles.some((style) => typeof style !== "string") ||
    !Number.isSafeInteger(value.page_revision_count) ||
    value.page_revision_count < 0 ||
    (value.page_updated_at !== null &&
      timestampParts(value.page_updated_at) === null)
  ) {
    throw new LocalPageReadError("invalid_page_result");
  }
  return Object.freeze({
    compiled_body_html: wellFormedString(value.compiled_body_html),
    compiled_body_styles: Object.freeze(
      value.compiled_body_styles.map(wellFormedString),
    ),
    page_revision_count: value.page_revision_count,
    page_updated_at: value.page_updated_at,
    wikitext: wellFormedString(value.wikitext),
  });
}

async function readResponseBytes(response) {
  const declaredLength = Number(response.headers.get("content-length"));
  if (
    Number.isFinite(declaredLength) &&
    declaredLength > MAX_LOCAL_RPC_RESPONSE_BYTES
  ) {
    throw new LocalPageReadError("response_too_large");
  }
  const reader = response.body?.getReader?.();
  if (reader === undefined) {
    const bytes = Buffer.from(await response.arrayBuffer());
    if (bytes.byteLength > MAX_LOCAL_RPC_RESPONSE_BYTES) {
      throw new LocalPageReadError("response_too_large");
    }
    return bytes;
  }
  const chunks = [];
  let total = 0;
  try {
    while (true) {
      const next = await reader.read();
      if (next.done) break;
      const chunk = Buffer.from(next.value);
      total += chunk.byteLength;
      if (total > MAX_LOCAL_RPC_RESPONSE_BYTES) {
        await reader.cancel().catch(() => {});
        throw new LocalPageReadError("response_too_large");
      }
      chunks.push(chunk);
    }
  } finally {
    reader.releaseLock();
  }
  return Buffer.concat(chunks, total);
}

export class LocalPageReadClient {
  #fetch;
  #nextId = 1;
  #rpcUrl;
  #timeoutMs;

  constructor({
    rpcUrl,
    timeoutMs = 30_000,
    fetchImpl = globalThis.fetch,
  } = {}) {
    if (
      !Number.isSafeInteger(timeoutMs) ||
      timeoutMs < 1 ||
      timeoutMs > 120_000
    ) {
      throw new Error(
        "local page read timeout must be an integer from 1 through 120000",
      );
    }
    if (typeof fetchImpl !== "function") {
      throw new Error("local page read fetch implementation is required");
    }
    this.#fetch = fetchImpl;
    this.#rpcUrl = validateLocalDeepwellRpcUrl(rpcUrl);
    this.#timeoutMs = timeoutMs;
  }

  async #call(method, params) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.#timeoutMs);
    try {
      const id = this.#nextId++;
      let response;
      try {
        response = await this.#fetch(this.#rpcUrl, {
          body: JSON.stringify({ jsonrpc: "2.0", id, method, params }),
          credentials: "omit",
          headers: { "content-type": "application/json" },
          method: "POST",
          redirect: "error",
          signal: controller.signal,
        });
      } catch (error) {
        throw new LocalPageReadError(
          error?.name === "AbortError" ? "timeout" : "transport",
        );
      }
      let bytes;
      try {
        bytes = await readResponseBytes(response);
      } catch (error) {
        if (error instanceof LocalPageReadError) throw error;
        throw new LocalPageReadError(
          error?.name === "AbortError" ? "timeout" : "transport",
        );
      }
      let body;
      try {
        body = JSON.parse(
          new TextDecoder("utf-8", { fatal: true }).decode(bytes),
        );
      } catch {
        throw new LocalPageReadError("invalid_response");
      }
      if (
        !response.ok ||
        body === null ||
        typeof body !== "object" ||
        Array.isArray(body)
      ) {
        throw new LocalPageReadError("rpc_response");
      }
      if (
        body.jsonrpc !== "2.0" ||
        body.id !== id ||
        Object.hasOwn(body, "error") ||
        !Object.hasOwn(body, "result")
      ) {
        throw new LocalPageReadError("rpc_envelope");
      }
      return body.result;
    } finally {
      clearTimeout(timer);
    }
  }

  async siteId() {
    const result = await this.#call("site_get", { site: "scp-wiki" });
    if (!Number.isSafeInteger(result?.site_id) || result.site_id < 1) {
      throw new LocalPageReadError("invalid_site_result");
    }
    return result.site_id;
  }

  async pageGet(siteId, fullname) {
    const result = await this.#call("page_get", {
      details: { compiled: true, wikitext: true },
      page: fullname,
      site_id: siteId,
    });
    return result === null ? null : localPage(result, { fullname, siteId });
  }
}
