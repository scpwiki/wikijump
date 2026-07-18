import assert from "node:assert/strict";
import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  AtomicPublicationAmbiguousError,
  publishBytesNoReplaceAt,
} from "../src/atomic-no-replace.mjs";

test(
  "publishes without replacement and classifies post-link fsync ambiguity",
  { skip: process.platform !== "linux" },
  async (t) => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "atomic-no-replace-"));
    t.after(() => fs.rm(root, { force: true, recursive: true }));
    const directoryHandle = await fs.open(
      root,
      fsConstants.O_RDONLY | fsConstants.O_DIRECTORY | fsConstants.O_NOFOLLOW,
    );
    t.after(() => directoryHandle.close());
    const bytes = Buffer.from("durability boundary");

    assert.equal(
      await publishBytesNoReplaceAt(directoryHandle, "normal", bytes),
      "created",
    );
    assert.equal(
      await publishBytesNoReplaceAt(
        directoryHandle,
        "normal",
        Buffer.from("different"),
      ),
      "exists",
    );
    assert.deepEqual(await fs.readFile(path.join(root, "normal")), bytes);

    const commitFailureHandle = {
      fd: directoryHandle.fd,
      sync: async () => {
        throw new Error("injected commit fsync failure");
      },
    };
    await assert.rejects(
      publishBytesNoReplaceAt(commitFailureHandle, "commit-failure", bytes),
      (error) =>
        error instanceof AtomicPublicationAmbiguousError &&
        error.published === true &&
        error.durable === false,
    );
    assert.deepEqual(
      await fs.readFile(path.join(root, "commit-failure")),
      bytes,
    );
    assert.equal(
      await publishBytesNoReplaceAt(directoryHandle, "commit-failure", bytes),
      "exists",
    );

    let syncCount = 0;
    const cleanupFailureHandle = {
      fd: directoryHandle.fd,
      sync: async () => {
        syncCount += 1;
        if (syncCount === 2) throw new Error("injected cleanup fsync failure");
      },
    };
    await assert.rejects(
      publishBytesNoReplaceAt(cleanupFailureHandle, "cleanup-failure", bytes),
      (error) =>
        error instanceof AtomicPublicationAmbiguousError &&
        error.published === true &&
        error.durable === true,
    );
    assert.deepEqual(
      await fs.readFile(path.join(root, "cleanup-failure")),
      bytes,
    );
    const temporaryNames = (await fs.readdir(root)).filter((name) =>
      name.endsWith(".tmp"),
    );
    assert.equal(temporaryNames.length, 1);
    await fs.unlink(path.join(root, temporaryNames[0]));
    await directoryHandle.sync();
    await assert.rejects(
      publishBytesNoReplaceAt(directoryHandle, "../escape", bytes),
      /safe path component/u,
    );
  },
);
