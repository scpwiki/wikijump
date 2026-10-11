/*
 * utils/slug.rs
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

use crate::utils::replace_in_place;
use wikidot_normalize::normalize;

/// Normalize a name to a slug. Does not preseve `:`.
///
/// Meant for use in sites and users.
pub fn normalize_slug_without_category_separator<S: Into<String>>(name: S) -> String {
    let mut slug = name.into();
    replace_in_place(&mut slug, ":", "-");
    normalize(&mut slug);
    slug
}

/// Normalize a name to a slug.
pub fn normalize_page_slug<S: Into<String>>(name: S) -> String {
    let mut slug = name.into();
    normalize(&mut slug);
    slug
}

/// Resolve only the page URL aliases observed in Wikidot's public router.
///
/// This intentionally does not use `normalize_page_slug`: that function also
/// applies Unicode compatibility normalization and rewrites arbitrary
/// punctuation, which is too broad for resolving an incoming URL to stored
/// content. The caller checks for an exact stored slug first, then looks up
/// this alias and applies the resolved page's normal permission checks.
pub fn observed_wikidot_page_url_alias(slug: &str) -> Option<String> {
    if slug.is_empty() || !slug.is_ascii() {
        return None;
    }

    let input = slug.trim_end_matches([' ', '.']).to_ascii_lowercase();
    if input.is_empty() {
        return None;
    }
    if matches!(input.as_str(), "forum" | "local--files" | "local--code") {
        return None;
    }

    let chars: Vec<_> = input.chars().collect();
    let mut colon_count = 0;
    let mut canonical = String::with_capacity(input.len());
    let mut at_segment_start = true;
    let mut observed_punctuation_aliases = 0;
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        match ch {
            ':' => {
                colon_count += 1;
                at_segment_start = true;
                canonical.push(ch);
            }
            'a'..='z' | '0'..='9' => {
                canonical.push(ch);
                at_segment_start = false;
            }
            '-' => {
                let run_end = chars[index..]
                    .iter()
                    .position(|next| *next != '-')
                    .map_or(chars.len(), |offset| index + offset);
                let run_length = run_end - index;

                // The source observation confirms one internal doubled
                // hyphen and one trailing hyphen. Leave leading, longer, and
                // trailing doubled runs unresolved.
                if at_segment_start
                    || run_length > 2
                    || (run_length == 2 && run_end == chars.len())
                    || (run_length == 2
                        && !matches!(chars.get(run_end), Some('a'..='z' | '0'..='9')))
                {
                    return None;
                }

                if run_length == 2 {
                    observed_punctuation_aliases += 1;
                }

                canonical.push('-');
                at_segment_start = false;
                index = run_end - 1;
            }
            '+' | '.' => {
                if at_segment_start
                    || canonical.ends_with('-')
                    || !matches!(chars.get(index + 1), Some('a'..='z' | '0'..='9'))
                {
                    return None;
                }
                observed_punctuation_aliases += 1;
                canonical.push('-');
                at_segment_start = false;
            }
            '_' => {
                canonical.push(if at_segment_start { '_' } else { '-' });
                at_segment_start = false;
            }
            _ => return None,
        }
        if colon_count > 1 {
            return None;
        }
        index += 1;
    }

    // A single trailing hyphen was observed to resolve to the unhyphenated
    // page slug. Do not trim a longer run (rejected above).
    if canonical.ends_with('-') && input.ends_with('-') {
        observed_punctuation_aliases += 1;
        canonical.pop();
    }

    if observed_punctuation_aliases > 1 {
        return None;
    }

    if matches!(canonical.as_str(), "forum" | "local--files" | "local--code") {
        return None;
    }

    if canonical != slug {
        Some(canonical)
    } else {
        None
    }
}

#[test]
fn regular_slug_replaces_category_separator() {
    assert_eq!(
        normalize_slug_without_category_separator("forum:staff"),
        "forum-staff"
    );
}

#[test]
fn page_slug_preserves_category_separator() {
    assert_eq!(normalize_page_slug("forum:staff"), "forum:staff");
}

#[test]
fn slug_normalization_handles_case_and_spacing() {
    assert_eq!(
        normalize_slug_without_category_separator("  Mixed Case  "),
        "mixed-case"
    );
    assert_eq!(normalize_page_slug("  Mixed Case  "), "mixed-case");
}

#[test]
fn observed_page_url_aliases_are_narrow_and_category_aware() {
    assert_eq!(
        observed_wikidot_page_url_alias("SCP-9506"),
        Some("scp-9506".to_owned())
    );
    assert_eq!(
        observed_wikidot_page_url_alias("scp_9506"),
        Some("scp-9506".to_owned())
    );
    assert_eq!(
        observed_wikidot_page_url_alias("scp-9506."),
        Some("scp-9506".to_owned())
    );
    assert_eq!(
        observed_wikidot_page_url_alias("scp+9506"),
        Some("scp-9506".to_owned())
    );
    assert_eq!(
        observed_wikidot_page_url_alias("scp.9506"),
        Some("scp-9506".to_owned())
    );
    assert_eq!(
        observed_wikidot_page_url_alias("scp--9506"),
        Some("scp-9506".to_owned())
    );
    assert_eq!(
        observed_wikidot_page_url_alias("scp-9506-"),
        Some("scp-9506".to_owned())
    );
    assert_eq!(
        observed_wikidot_page_url_alias("scp-9506 "),
        Some("scp-9506".to_owned())
    );
    assert_eq!(
        observed_wikidot_page_url_alias("SYSTEM:JOIN"),
        Some("system:join".to_owned())
    );
    assert_eq!(
        observed_wikidot_page_url_alias("_TEMPLATE"),
        Some("_template".to_owned())
    );
    assert_eq!(observed_wikidot_page_url_alias("scp---9506"), None);
    assert_eq!(observed_wikidot_page_url_alias("-scp-9506"), None);
    assert_eq!(observed_wikidot_page_url_alias("scp-9506--"), None);
    assert_eq!(observed_wikidot_page_url_alias("scp-.9506"), None);
    assert_eq!(observed_wikidot_page_url_alias("scp+9506-"), None);
    assert_eq!(observed_wikidot_page_url_alias("local--files"), None);
    assert_eq!(observed_wikidot_page_url_alias("local--code"), None);
    assert_eq!(observed_wikidot_page_url_alias("local++files"), None);
    assert_eq!(observed_wikidot_page_url_alias("scp/%2fadmin"), None);
    assert_eq!(observed_wikidot_page_url_alias("ѕср-9506"), None);
}
