/*
 * data/user_info.rs
 *
 * ftml - Library to parse Wikidot text
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

use super::KarmaLevel;
use std::borrow::Cow;

/// Returned information about a user.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct UserInfo<'a> {
    /// The user ID.
    ///
    /// This uniquely identifies a user even if they later change their name.
    pub user_id: i64,

    /// The user slug.
    pub user_slug: Cow<'a, str>,

    /// The user's display name.
    pub user_name: Cow<'a, str>,

    /// The user's karma, from 0-5.
    pub user_karma: KarmaLevel,

    /// Inline image data.
    ///
    /// Must be a valid [data URI] containing image data.
    ///
    /// [data URI]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs
    pub user_avatar_data: Cow<'a, str>,

    /// The link pointing to the user's information page.
    pub user_profile_url: Cow<'a, str>,
}

impl UserInfo<'_> {
    // TODO Add #[cfg(test)]

    /// Generate a dummy UserInfo instance for tests.
    pub fn dummy() -> Self {
        const AVATAR_BASE64_DATA: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAYAAABXAvmHAAADPElEQVRoBe1Yz0s6QRSf/frtIFpQGXqQik6GG9atg167iYdEunTv2FXoH+hueujsuUsQdAg7FFSHQDqI5ElQTPCgotRuTozg0PxYfa59v7vBDiy77zPz3nw++96M7igYY4x+cfvzi7kPqTsCrM6gkwEnAzO+ActK6PPzE62trSFFUYbXycmJOSnkd8Cqdnp6Sn6DhpfL5cL39/dTU0FTe/ygQ6vVwm63m4pQVRVrmjbVDJaVEKmXxcVFtL+/T0vn5eUFnZ2dURv0MJXcfzC4UCjQDJBy8vl8uNvtgmeytIQIy8FggIPBICOCrA1os7SESImQXSiVSjHVkslkENmlIA0kYLTVfb9DgkPHxONxZmi1WkWXl5cMZmhAUjXa6r7fIX7QMR8fH9jr9TJllEqlQO6gDBiq/6GOubk5tL29zUS7urpC7+/vDCYzbCGAEFNVleHX6XTQw8MDg8kM2wjY2toS+N3d3QkYD/zlAZn9P746w+GwMHWxWBQwHrBNBpaXl3luqFKpCBgPgDLAO42zyVYLaXxWFxYWBLd6vS5gPGCbDMgEdLtdnq9g20bA/Py8QE7TNAHjAdsIkJH1eDw8X8G2jYB+vy+Qky1sftCPL2J+cfITGtlvb29C1+rqqoDxgG0yUKvVeG5oc3NTwHgAlAHZ1mj2TfMERna5XB490vvOzg59NnqwTQZKpZLAMRqNChgP2EbA09MTw219fR1tbGwwmMwwJUBWUrLgUEzXdfT8/MwMTyQSjG1kmBIA2Z+NJpThj4+PqNfrMV0HBweMbWSABJAFS65AIDCMs7S0ZBTPFH5zc8P4kd1nd3eXwYwMkADiTP5YNRqNYZxIJGIUzxR+cXHB+B0fHzP2WAPy4UmOPg4PD+k3az6fh7iBxry+vtK45Jvb7/fjXq8H8iWDJp4LlctlHIvF6CR7e3tY13XwBJMGptNpGpsIyOVyk1yY/okCms0mPb9MJpO43W4zAWYx+v0+XllZoQJCodDUZ6MTBRCC2WwWX19fz8JV6nt+fk7JK4qCb29vpePGgSAB4wKY7SPrirzx0VnT0dGRqVAK8Rq7ym3eCd5G7arDEWB1ZpwMOBmY8Q38+hL6AuHLUi2wzjYWAAAAAElFTkSuQmCC";

        UserInfo {
            user_id: 0,
            user_slug: cow!("michal-frackowiak"),
            user_name: cow!("Michal Frackowiak"),
            user_karma: KarmaLevel::new(5).unwrap(),
            user_avatar_data: cow!(AVATAR_BASE64_DATA),
            user_profile_url: cow!("/user:info/michal-frackowiak"),
        }
    }
}
