# Deviation: Wikidot Redirect location recognition

## Shim

`wikidot_redirect_location` in `deepwell/src/services/view/redirect.rs` recognizes a preserved Wikidot Redirect module using `REDIRECT_MODULE_PREFIX_REGEX`, `REDIRECT_MODULE_REGEX`, and `REDIRECT_ARGUMENT_REGEX`. The scanner extracts only the authored destination needed by the runtime and fails closed for ambiguous or unsupported shapes.

## Reason

Turning an authored Redirect module into a response is runtime behavior, but Deepwell currently has to recognize the module syntax before it can apply that behavior. Keeping this bounded scanner next to page-view resolution lets Wikijump preserve the evidenced redirect contract without adding site or request state to FTML.

HTTP status selection, site routing, permission checks, external-destination policy, loop handling, request arguments, and every other runtime semantic remain owned by Wikijump. This deviation covers only temporary syntax recognition.

## Why FTML is not yet sufficient

The pinned FTML revision `1ed821a4e5cd1624310daf1bc911b0f986103c92` has no delayed Redirect representation that preserves a structured destination for caller resolution. FTML leaves the module in rendered or preserved source form, so Deepwell cannot resolve the runtime action without temporarily reparsing that source.

## Evidence

The frozen 40-page comparison at `/home/roku/wjlab/evidence/unknowns-surge-20260716/t1-templates-redirects/redirect-http-comparison-summary.json`, covered by the checksums in the same evidence directory, records 40 of 40 real Wikidot pages returning HTTP 301 with a Location header while the standing Wikijump runtime returned HTTP 200 without Location for all 40. The associated report at `/home/roku/codex-thread-workspaces/019f5be0-14ef-76b2-8319-229f06cbf145/artifacts/p0-runtime-parity-20260716/report.json` records the bounded Deepwell recognition and Wikijump-owned routing implementation.

## FTML backlog decision

Accept this scanner as new, bounded Wikijump-side syntax debt until FTML exposes a preserved delayed Redirect node. The FTML backlog item is to add that node with the raw module boundary and structured destination while leaving response, site, permission, routing, and destination-policy decisions to the caller.

## Migration condition

Remove `wikidot_redirect_location`, `REDIRECT_MODULE_PREFIX_REGEX`, `REDIRECT_MODULE_REGEX`, and `REDIRECT_ARGUMENT_REGEX` when the pinned FTML version exposes a preserved Redirect node that Deepwell can consume without reparsing source and the 40-page redirect contract plus malformed-input tests prove equivalent fail-closed behavior.

## Owner

Rokurolize/Wikijump maintainers own this temporary scanner and the matching FTML backlog item.

## Review trigger

Re-evaluate on every FTML pin bump, when FTML adds or changes Redirect module parsing, or before extending any accepted Redirect syntax shape in Deepwell.
