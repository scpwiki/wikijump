# Runtime Drift Policy

This policy governs the local standing Wikijump runtime used for continuous browser and client compatibility checks. Its purpose is to make every accepted observation attributable to one merged source revision and to prevent an old candidate from silently becoming the browser-facing service.

## Authoritative standing runtime

Port 443 is owned by exactly one standing Compose stack. That stack is built only from the current merged `develop` head and its committed FTML lock pin. It is not a branch checkout, a pull-request image, a local dirty tree, or a retained test container.

Before a standing runtime is accepted, record the merged Wikijump SHA and tree, FTML SHA, image digests for the gateway and every application upstream, profile and feature set where applicable, and the artifact key for any compiled Deepwell binary. The recorded gateway-to-upstream chain is the identity of the service, not the image tag alone.

The standing stack must remain available as a deliverable. Its normal canaries cover HTTP responses for representative pages and assets, WIKIREQUEST metadata, AJAX ListPages, DOM rendering, and an unmodified `wikidot.py` site and page lookup. A failed canary is a runtime incident and is repaired before unrelated candidate work proceeds.

## Candidate runtimes

Candidate stacks are isolated from the standing stack and never publish port 443. Each candidate has an explicit owner, Wikijump SHA and tree, FTML SHA, profile, artifact key or image digest, creation time, expiry time, and evidence directory. Candidate names, Compose project names, container labels, and receipts must carry that identity.

Candidates may use a dedicated non-443 port or no host port. They must not share the authoritative service name or network alias while the standing stack is serving traffic. A candidate is stopped and archived or removed when its receipt is terminal or its expiry passes. Retaining an image or volume is not evidence that a candidate is still authoritative.

## Candidate image lifecycle

Candidate images are disposable build outputs, not durable provenance. When a candidate is superseded or its verdict receipt becomes terminal, remove its containers and images in the same closure step after proving that no live container references them. The closure receipt records the image digest, candidate owner and identity, container-reference check, removal result, and the retained receipt path. If a historical candidate is needed again, rebuild it from the receipt's source revision, FTML pin, artifact key, profile, and features instead of relying on an old local image.

The current production image and one immediate rollback image are explicitly retained through a successful promotion. They are standing-runtime recovery assets, not candidate retention. They may be removed only after a later promotion receipt confirms a newer healthy production image and replacement rollback image. Image cleanup must never remove a digest referenced by the live standing stack, the immediate rollback stack, or a nonterminal candidate receipt.

## Promotion after merge

Promotion begins only after the candidate change has merged normally to `develop`. Rebuild from that exact merged head rather than reusing a branch image. Verify the source tree, FTML pin, image digests, and compiled artifact identity before changing the standing stack.

The switch is atomic from the browser's perspective: park the old standing application containers, start the exact merged-head containers with the authoritative aliases, wait for health, and then run the standing canaries. If the new stack fails health or any canary, restore the parked known-good stack and record the failed promotion receipt. Do not leave a candidate under the standing container name after a failed or interrupted promotion.

The promotion receipt records URL to gateway to upstream image digest to Wikijump SHA to FTML SHA, the old and new container identities, the canary commands and results, the final port-443 owner, and the retained production and rollback image digests. The old stack can be archived only after the receipt confirms the new stack is healthy.

## Measurements and maintenance windows

Browser-facing measurement work is normally performed against an isolated candidate. If a measurement must temporarily use the standing routing identity, the browser edges return an explicit 503 maintenance response for the bounded interval. The measurement wrapper owns restoration, verifies the exact production image after restoration, and reruns the standing canaries before releasing its lease. A candidate must never be served as a substitute for the standing runtime during that interval.

## Drift response

When a source checkout, container name, image digest, FTML pin, or canary result disagrees with the standing receipt, stop accepting evidence from that runtime. Recover the authoritative stack from the latest merged-head promotion receipt, then issue a compact incident receipt describing the observed identity, the expected identity, the owner if known, and the repair result. Do not infer safety from an HTTP 200 alone.
