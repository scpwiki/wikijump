/*
 * url.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

pub fn url_valid(url: &str) -> bool {
    const SCHEMES: [&str; 20] = [
        "blob:",
        "chrome-extension://",
        "chrome://",
        "content://",
        "data:",
        "dns:",
        "feed:",
        "file://",
        "ftp://",
        "git://",
        "gopher://",
        "http://",
        "https://",
        "irc6://",
        "irc://",
        "ircs://",
        "mailto:",
        "resource://",
        "rtmp://",
        "sftp://",
    ];

    // If url is an empty string
    if url.is_empty() {
        return false;
    }

    // If it's a relative link
    if url.starts_with('/') {
        return true;
    }

    // If it's a URL
    for scheme in &SCHEMES {
        if url.starts_with(scheme) {
            return true;
        }
    }

    false
}
