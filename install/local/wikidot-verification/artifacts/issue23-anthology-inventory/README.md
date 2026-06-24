# Issue #23 / SCP Anthology 2024 resource profile

This directory contains the in-repository resource inventory artifact generated for Issue #23.

Included files:
- `anthology-resource-profile.tsv`: tab-separated resource inventory with the columns `resource_type`, `value`, `reference`, and `context`.

The inventory was produced from the local SCP Anthology 2024 corpus source and related child-page metadata, then checked into this repository so later resource-sync work does not depend on a controller-specific scratch path.

This artifact is inventory only. It does not download remote resources, expand `wdfiles.com` support, or prove browser-visible local resource serving.
