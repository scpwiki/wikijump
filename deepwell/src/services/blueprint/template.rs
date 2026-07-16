/*
 * services/blueprint/template.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::borrow::Cow;

const CONTENT_PLACEHOLDER: &str = "%%content%%";

/// Returns the only page template that may apply to this page.
///
/// Wikidot applies `_template` to the default category and
/// `{category}:_template` to a named category. Named categories do not fall
/// back to the default template, and template pages do not apply themselves.
pub(super) fn exact_template_slug<'a>(
    category: Option<&'a str>,
    page: &str,
    template_page: &'a str,
) -> Option<Cow<'a, str>> {
    if page == template_page {
        return None;
    }

    Some(match category {
        Some(category) => Cow::Owned(format!("{category}:{template_page}")),
        None => Cow::Borrowed(template_page),
    })
}

/// Places raw page source into the observed Wikidot template placeholder.
///
/// The oracle contract currently covers one placeholder. Replacing only its
/// first occurrence keeps additional, unverified occurrences literal.
pub(super) fn compose_template(template: &str, content: &str) -> String {
    template.replacen(CONTENT_PLACEHOLDER, content, 1)
}

#[cfg(test)]
mod tests {
    use super::{compose_template, exact_template_slug};

    #[test]
    fn selects_only_the_exact_category_template() {
        assert_eq!(
            exact_template_slug(None, "article", "_template").as_deref(),
            Some("_template"),
        );
        assert_eq!(
            exact_template_slug(Some("scp"), "article", "_template").as_deref(),
            Some("scp:_template"),
        );
        assert_eq!(exact_template_slug(None, "_template", "_template"), None);
        assert_eq!(
            exact_template_slug(Some("scp"), "_template", "_template"),
            None,
        );
    }

    #[test]
    fn composes_the_observed_single_placeholder_without_inference() {
        assert_eq!(
            compose_template("BEFORE\n%%content%%\nAFTER", "BODY"),
            "BEFORE\nBODY\nAFTER",
        );
        assert_eq!(compose_template("NO PLACEHOLDER", "BODY"), "NO PLACEHOLDER");
        assert_eq!(
            compose_template("%%content%%|%%content%%", "BODY"),
            "BODY|%%content%%",
        );
    }
}
