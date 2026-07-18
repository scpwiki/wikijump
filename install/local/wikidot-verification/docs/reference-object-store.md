# Reference object store v1

The reference object store is the language-neutral immutable byte store for Wikidot XML-RPC, HTTP, browser, resource, and visual evidence. It has no acquisition policy and does not determine whether an observation is correct. Later receipts bind objects to an acquisition inventory, producer contract, media type, role, and result.

## Normative files

[`fixtures/reference-object-store-v1/store.json`](../fixtures/reference-object-store-v1/store.json) contains the exact `store.json` bytes: 285 UTF-8 bytes, no BOM, compact lexicographically ordered keys, and one terminal LF. Its SHA-256 is `dfc3db9423713751f1f8bda474b934632fa969232f6a44dabb28e765a6288f79`. Opening a store requires an exact byte match.

[`schemas/reference-object-store-v1.schema.json`](../schemas/reference-object-store-v1.schema.json) validates the descriptor. [`schemas/reference-object-v1.schema.json`](../schemas/reference-object-v1.schema.json) independently validates an object reference. Their versioned `$id` values are immutable contract identifiers.

[`fixtures/reference-object-store-v1/vectors.json`](../fixtures/reference-object-store-v1/vectors.json) is the normative conformance input for Node, Python, and later producers. It covers empty, text with and without a terminal LF, non-ASCII UTF-8, and arbitrary binary bytes.

## Object identity and references

SHA-256 covers the exact stored bytes. Producers must not decode, normalize newlines, recompress, or reserialize content before hashing unless a later receipt explicitly defines that derived representation.

An object reference contains exactly `algorithm`, `bytes`, and `sha256`. `algorithm` is `sha256`; `bytes` is the exact octet count from zero through 2^53−1; `sha256` is 64 lowercase hexadecimal characters. The path is `objects/sha256/<first two digest characters>/<full digest>`. References never contain a host path, URL, MIME type, filename, timestamp, or producer identity.

## Publication and corruption

Writers stage a private temporary file in the destination directory, write and fsync it, set mode `0400`, then hard-link it to the digest path. Hard-link creation is the no-replace publication point. A competing writer accepts `EEXIST` only after opening the existing regular file without following symlinks and rehashing all bytes. Writers never replace, truncate, repair, quarantine, or delete a digest path.

Every newly created directory entry and published object is committed by fsyncing its pinned parent directory. A failure after a link becomes visible is an ambiguous publication, not permission to overwrite: retry and verify the digest path. Corrupt, truncated, symlinked, non-regular, moved, or rebound store components fail closed.

The current Node writer profile intentionally requires Linux descriptor-relative traversal through `/proc/self/fd`, same-filesystem hard links, same-UID cooperative producers, directories at mode `0700`, and descriptor/object files at mode `0400`. These permissions are read-only policy, not kernel immutability. Other-language writers may implement the same on-disk contract with native `openat`/`linkat` equivalents, but evidence production must not fall back to pathname-racy or overwrite-capable publication.

Recovery restores exact trusted bytes or creates a new store generation; it does not mutate a poisoned object in place. Shard leases and garbage collection are separate later contracts.

## Deterministic acquisition completion

`completions/index.json` describes the auxiliary immutable completion index. Its leaves are canonical completion-pointer JSONL files at `completions/sha256/<first two work-digest characters>/<full work digest>`. The frozen `store.json` contract remains unchanged.

A work digest binds the externally pinned inventory row, requested layer, and producer contract object. A pointer counts as complete only after its attempt receipt, producer object, and every evidence object verify transitively and the receipt has a complete outcome for that exact work digest. A missing leaf is pending. A pointer to a failed attempt, malformed or noncanonical pointer, wrong-work receipt, dangling object, corrupt object, symlink, non-regular file, or rebound directory is an error rather than resumable absence. An unindexed failed receipt leaves its work pending.

Publication is first-valid-writer-wins through the same no-replace hard-link primitive as CAS objects. Repeating the same receipt is idempotent. A different valid complete receipt for an occupied work digest is an explicit conflict, and the original pointer remains authoritative. Resume derives every expected work target in frozen inventory row and requested-layer order, groups only those expected digests by two-character prefix for descriptor-relative bulk reads, and restores row/layer order before semantic verification; it never scans arbitrary directory entries or treats poisoned state as pending. Completion pointers are derived indexes; attempt receipts and their content-addressed objects remain the durable evidence.
