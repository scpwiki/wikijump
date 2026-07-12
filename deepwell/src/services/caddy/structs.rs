/*
 * services/caddy/structs.rs
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

use sea_orm::FromQueryResult;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

#[derive(Deserialize, Clone)]
pub struct CaddyfileOptions<'a> {
    /// Whether to enable debug logging in Caddy.
    ///
    /// Corresponds to the Caddy option `debug`.
    #[serde(default)]
    pub debug: bool,

    /// Whether to operate in local mode.
    ///
    /// This uses local self-signed certificates and
    /// attempts no ACME challenges because the server
    /// is not running in an environment where it actually
    /// owns a domain name.
    ///
    /// Corresponds to Caddy options `local_certs` and `skip_install_trust`.
    #[serde(default)]
    pub local: bool,

    /// The HTTP port to use.
    ///
    /// Corresponds to the Caddy option `http_port`.
    #[serde(default)]
    pub http_port: Option<u16>,

    /// The HTTPS port to use.
    ///
    /// Corresponds to the Caddy option `https_port`.
    #[serde(default)]
    pub https_port: Option<u16>,

    /// TLS wildcard certificate settings.
    ///
    /// This option, if `Some(_)`, instructs Caddy to perform
    /// a DNS ACME challenge to generate a wildcard certificate
    /// for the fallback cases.
    ///
    /// The value here is used in a `dns` directive to instruct
    /// Caddy which provider to use to perform the ACME challenge.
    ///
    /// For instance, the value might be `digitalocean {env.DIGITALOCEAN_TOKEN}`
    /// to use the [DigitalOcean provider](https://github.com/caddy-dns/digitalocean).
    ///
    /// If this option is `None`, then it means that no wildcard
    /// certificates should be fetched for this server. In this case,
    /// all unknown domains and `wjfiles` are handled as HTTP.
    #[serde(default)]
    pub wildcard_cert: Option<Cow<'a, str>>,

    /// Enables a `deploy` subdomain redirect.
    ///
    /// If this is running in an environment where a system
    /// like [Komodo](https://komo.do/) is responsible for
    /// deploying Wikijump.
    ///
    /// This should have the host of the deployment system's
    /// web server to reverse proxy to.
    #[serde(default)]
    pub deploy_host: Option<Cow<'a, str>>,

    /// Specifies the framerail host to reverse proxy to.
    pub framerail_host: Cow<'a, str>,

    /// Specifies the WWS host to reverse proxy to.
    pub wws_host: Cow<'a, str>,
}

impl fmt::Debug for CaddyfileOptions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CaddyfileOptions")
            .field("debug", &self.debug)
            .field("local", &self.local)
            .field("http_port", &self.http_port)
            .field("https_port", &self.https_port)
            .field(
                "wildcard_cert",
                &self.wildcard_cert.as_ref().map(|_| "[redacted]"),
            )
            .field("deploy_host", &self.deploy_host)
            .field("framerail_host", &self.framerail_host)
            .field("wws_host", &self.wws_host)
            .finish()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SiteData {
    pub sites: Vec<(i64, String, Option<String>)>,
    pub domains: HashMap<i64, SiteDomainData>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct SiteDomainData {
    pub aliases: Vec<String>,
    pub custom_domains: Vec<CustomDomainData>,
}

#[derive(Deserialize, FromQueryResult, Debug, Default, Clone)]
pub struct CustomDomainData {
    pub domain: String,
    pub www_redirect: bool,
}
