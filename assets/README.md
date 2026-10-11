## Assets

This directory is reserved for redistributable Wikijump project assets.

Do not add logo or brand files here unless their repository redistribution license is documented and compatible with downstream source distribution.

## Repository-authored compatibility assets

`feed-icon-14x14.svg` is the 14x14 RSS chrome icon used by Wikidot-compatible feed links. It was drawn in this repository from geometry (an orange rounded square with a white dot and two arcs) and is not a copy of any Wikidot-hosted file. It is covered by the repository licence (AGPL-3.0, Wikijump Team).

- Source: `assets/feed-icon-14x14.svg`, sha256 `8400559ef9d186d1d50671f63193b2e9343d0e26a03d1cd3232735ccbd786576`.
- Render: `convert -background none -density 1152 feed-icon-14x14.svg -resize 14x14 -strip png32:out.png` (ImageMagick 6.9.12), producing sha256 `332f1155f4635f7d79797ac40141990c62a140921feb68bc3d3d7a1bfa7ab8c6`.
- Served bytes: `framerail/src/lib/server/wikidot-feed-icon.ts`, exposed by the allowlisted route `framerail/src/routes/common--theme/base/images/feed/feed-icon-14x14.png/+server.ts`. The unit test `framerail/tests/wikidot-feed-icon.test.ts` pins both hashes.
