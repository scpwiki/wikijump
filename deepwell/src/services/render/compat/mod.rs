pub(super) mod color_and_inline_protection;
mod fallback_code;
mod fallback_render;
pub(super) mod footnote_dom;
mod html_fragments;
pub(super) mod issued_markers;
pub(super) mod preparation;
pub(super) mod text_fragments;
pub(super) mod wikidot_compat_restore;
pub(super) mod wikidot_embed;
pub(super) mod wikidot_inline_markers;
pub(super) mod wikidot_link_protection;
pub(super) mod wikidot_residual_markers;

pub(super) use self::color_and_inline_protection::sanitize_wikidot_compat_inline_tag;
pub(super) use self::fallback_code::{
    WikidotCompatibilityFallbackOutput, scan_compat_code_blocks,
};
pub(super) use self::html_fragments::CompatHtmlFragments;
