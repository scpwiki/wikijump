/*
 * services/render/wikidot_hosts.rs
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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Reading a site slug back out of a Wikidot-shaped host name.
//!
//! Imported content refers to sites by host: `example.wikidot.com` for pages
//! and `example.wdfiles.com` for uploaded files. The renderer has to map those
//! back onto local sites, and each of these helpers answers one part of that.

use crate::config::Config;

pub(super) fn preferred_domain_wikidot_slug(preferred_domain: &str) -> Option<String> {
    let host = preferred_domain_host(preferred_domain)?;
    let wikidot_slug = host.strip_suffix(".wikidot.com").unwrap_or(host.as_str());

    (!wikidot_slug.is_empty()).then(|| wikidot_slug.to_owned())
}

pub(super) fn preferred_domain_host(preferred_domain: &str) -> Option<String> {
    let trimmed = preferred_domain.trim().trim_end_matches('.');
    let host = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed)
        .split('/')
        .next()
        .unwrap_or(trimmed)
        .split(':')
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches('.');

    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

pub(super) fn local_file_host_site_slug(host: &str, config: &Config) -> Option<String> {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();

    [".wikidot.com", ".wdfiles.com", ".wjfiles.com"]
        .iter()
        .find_map(|suffix| host.strip_suffix(suffix).map(ToOwned::to_owned))
        .filter(|slug| !slug.is_empty())
        .or_else(|| {
            let suffix = config.files_domain.to_ascii_lowercase();
            host.strip_suffix(&suffix)
                .map(ToOwned::to_owned)
                .filter(|slug| !slug.is_empty())
        })
}

pub(super) fn direct_wdfiles_local_file_url(host: &str, path: &str) -> Option<String> {
    if !path.starts_with("/local--files/") {
        return None;
    }

    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    let site_slug = host.strip_suffix(".wikidot.com")?;
    if site_slug.is_empty()
        || !site_slug
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
        || !site_slug
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        || !site_slug
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
    {
        return None;
    }

    Some(format!("https://{site_slug}.wdfiles.com{path}"))
}

#[allow(dead_code)]
pub(super) fn public_url_port_suffix(port: Option<u16>) -> String {
    port.map(|port| format!(":{port}")).unwrap_or_default()
}
