import { publishBytesNoReplaceAt } from "./atomic-no-replace.mjs";
import { stableStringify } from "./canonical-json.mjs";

const SHA256_RE = /^[0-9a-f]{64}$/u;
const INDEX_READ_BATCH_SIZE = 16;
export const REFERENCE_COMPLETION_POINTER_MAX_BYTES = 1024;

const COMPLETION_INDEX_DESCRIPTOR = Object.freeze({
  digest_encoding: "lowercase-hex",
  hash_algorithm: "sha256",
  pointer_encoding: "stable-json-v1-jsonl",
  pointer_path_template: "sha256/{prefix2}/{work_identity_sha256}",
  pointer_schema:
    "https://wikijump.org/schemas/reference-acquisition-completion-pointer-v1.schema.json",
  schema: "wikijump_full_parity.reference_acquisition_completion_index.v1",
});
const COMPLETION_INDEX_DESCRIPTOR_BYTES = Buffer.from(
  `${stableStringify(COMPLETION_INDEX_DESCRIPTOR)}\n`,
);

function assertCompletionDigest(value) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error("work identity must be a lowercase SHA-256 digest");
  }
}

function visibleRecord(bytes) {
  return Object.freeze({ bytes });
}

function errorRecord(error) {
  return Object.freeze({ error });
}

class ReferenceCompletionIndex {
  #handles;

  constructor(handles) {
    this.#handles = handles;
  }

  async #assertBindings() {
    const handles = this.#handles;
    if (handles === null) {
      throw new Error("reference completion index is closed");
    }
    await handles.assertStoreBindings();
    await handles.assertDirectoryBinding(
      handles.root,
      "completions",
      handles.completions,
      "completions",
    );
    await handles.assertDirectoryBinding(
      handles.completions,
      "sha256",
      handles.sha256,
      "completion sha256",
    );
    return handles;
  }

  async #openPrefix(handles, name, create, recheckMissing = true) {
    try {
      return await handles.openDirectoryAt(handles.sha256, name, {
        create,
        label: `completion prefix ${name}`,
      });
    } catch (error) {
      if (!create && error.code === "ENOENT") {
        if (recheckMissing) await this.#assertBindings();
        return null;
      }
      throw error;
    }
  }

  #readVisible(handles, prefix, digest, allowMissing = false) {
    return handles.readAndHashFileAt(prefix, digest, {
      allowMissing,
      collect: true,
      expectedMode: 0o400,
      maxBytes: REFERENCE_COMPLETION_POINTER_MAX_BYTES,
      sizeMismatchMessage: `completion ${digest} exceeds its byte limit`,
    });
  }

  async read(digest) {
    assertCompletionDigest(digest);
    const handles = await this.#assertBindings();
    const prefixName = digest.slice(0, 2);
    const prefix = await this.#openPrefix(handles, prefixName, false);
    if (prefix === null) return null;
    try {
      const visible = await this.#readVisible(handles, prefix, digest, true);
      await handles.assertDirectoryBinding(
        handles.sha256,
        prefixName,
        prefix,
        `completion prefix ${prefixName}`,
      );
      await this.#assertBindings();
      return visible?.contents ?? null;
    } finally {
      await prefix.close();
    }
  }

  async readMany(digests) {
    if (!Array.isArray(digests)) {
      throw new Error("completion digests must be an array");
    }
    for (const digest of digests) assertCompletionDigest(digest);
    const records = new Array(digests.length);
    const groups = new Map();
    for (let index = 0; index < digests.length; index += 1) {
      const digest = digests[index];
      const prefixName = digest.slice(0, 2);
      let entries = groups.get(prefixName);
      if (entries === undefined) {
        entries = [];
        groups.set(prefixName, entries);
      }
      entries.push({ digest, index });
    }
    const handles = await this.#assertBindings();
    for (const [prefixName, entries] of groups) {
      let prefix;
      try {
        prefix = await this.#openPrefix(handles, prefixName, false, false);
      } catch (error) {
        for (const entry of entries) records[entry.index] = errorRecord(error);
        await this.#assertBindings();
        continue;
      }
      if (prefix === null) {
        for (const entry of entries) records[entry.index] = visibleRecord(null);
        await this.#assertBindings();
        continue;
      }
      try {
        for (
          let offset = 0;
          offset < entries.length;
          offset += INDEX_READ_BATCH_SIZE
        ) {
          const batch = entries.slice(offset, offset + INDEX_READ_BATCH_SIZE);
          const settled = await Promise.allSettled(
            batch.map((entry) =>
              this.#readVisible(handles, prefix, entry.digest, true),
            ),
          );
          for (let index = 0; index < settled.length; index += 1) {
            const result = settled[index];
            const entry = batch[index];
            records[entry.index] =
              result.status === "rejected"
                ? errorRecord(result.reason)
                : visibleRecord(result.value?.contents ?? null);
          }
        }
        await handles.assertDirectoryBinding(
          handles.sha256,
          prefixName,
          prefix,
          `completion prefix ${prefixName}`,
        );
      } catch (error) {
        for (const entry of entries) records[entry.index] = errorRecord(error);
      } finally {
        await prefix.close();
      }
      await this.#assertBindings();
    }
    await this.#assertBindings();
    return Object.freeze(records);
  }

  async publish(digest, value) {
    assertCompletionDigest(digest);
    const bytes = Buffer.from(value);
    if (bytes.byteLength > REFERENCE_COMPLETION_POINTER_MAX_BYTES) {
      throw new Error("completion pointer exceeds its byte limit");
    }
    const handles = await this.#assertBindings();
    const prefixName = digest.slice(0, 2);
    const prefix = await this.#openPrefix(handles, prefixName, true);
    try {
      const disposition = await publishBytesNoReplaceAt(prefix, digest, bytes, {
        mode: 0o400,
      });
      if (disposition === "exists") await prefix.sync();
      const visible = await this.#readVisible(handles, prefix, digest);
      await handles.assertDirectoryBinding(
        handles.sha256,
        prefixName,
        prefix,
        `completion prefix ${prefixName}`,
      );
      await this.#assertBindings();
      return Object.freeze({ bytes: visible.contents, disposition });
    } finally {
      await prefix.close();
    }
  }

  async close() {
    const handles = this.#handles;
    this.#handles = null;
    if (handles === null) return;
    const results = await Promise.allSettled([
      handles.sha256.close(),
      handles.completions.close(),
    ]);
    const failure = results.find((result) => result.status === "rejected");
    if (failure !== undefined) throw failure.reason;
  }
}

export async function prepareReferenceCompletionIndex(root, options) {
  await options.assertStoreBindings();
  let completions;
  let sha256;
  try {
    completions = await options.openDirectoryAt(root, "completions", {
      create: options.create,
      label: "completions",
    });
    sha256 = await options.openDirectoryAt(completions, "sha256", {
      create: options.create,
      label: "completion sha256",
    });
    if (options.create) {
      await publishBytesNoReplaceAt(
        completions,
        "index.json",
        COMPLETION_INDEX_DESCRIPTOR_BYTES,
        { mode: 0o400 },
      );
    }
    const descriptor = await options.readAndHashFileAt(
      completions,
      "index.json",
      {
        collect: true,
        expectedMode: 0o400,
        maxBytes: COMPLETION_INDEX_DESCRIPTOR_BYTES.byteLength,
        sizeMismatchMessage: "completion index descriptor is not canonical",
      },
    );
    if (!descriptor.contents.equals(COMPLETION_INDEX_DESCRIPTOR_BYTES)) {
      throw new Error("completion index descriptor is not canonical");
    }
    await completions.sync();
    await options.assertDirectoryBinding(
      root,
      "completions",
      completions,
      "completions",
    );
    await options.assertDirectoryBinding(
      completions,
      "sha256",
      sha256,
      "completion sha256",
    );
    await options.assertStoreBindings();
    return new ReferenceCompletionIndex({
      ...options,
      completions,
      root,
      sha256,
    });
  } catch (error) {
    await sha256?.close().catch(() => {});
    await completions?.close().catch(() => {});
    throw error;
  }
}
