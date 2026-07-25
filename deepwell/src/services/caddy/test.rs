/*
 * services/caddy/test.rs
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

//! Unit testing for our generated `Caddyfile`s.

use super::structs::{CaddyfileOptions, CustomDomainData, SiteData, SiteDomainData};
use crate::config::Config;
use crate::services::CaddyService;
use maplit::hashmap;
use pretty_assertions::assert_eq;
use std::fs::File;
use std::io::{Read, Write};

fn build_config(main_domain: &str, files_domain: &str) -> Config {
    assert!(!main_domain.starts_with('.'));
    assert!(!files_domain.starts_with('.'));

    let mut config = Config::integration_testing();
    config.main_domain_no_dot = str!(main_domain);
    config.main_domain = format!(".{main_domain}");
    config.files_domain_no_dot = str!(files_domain);
    config.files_domain = format!(".{files_domain}");
    config
}

fn build_site_data() -> (SiteData, SiteData) {
    macro_rules! domain {
        ($domain:expr $(,)?) => {
            domain!($domain, true)
        };

        ($domain:expr, $www_redirect:expr $(,)?) => {
            CustomDomainData {
                domain: str!($domain),
                www_redirect: $www_redirect,
            }
        };
    }

    let basic = SiteData {
        sites: vec![
            (1, str!("foo"), None),
            (2, str!("bar"), Some(str!("example.com"))),
        ],
        domains: hashmap! {
            1 => SiteDomainData::default(),
            2 => SiteDomainData {
                aliases: vec![],
                custom_domains: vec![domain!("example.com")],
            },
        },
    };

    let full = SiteData {
        sites: vec![
            (1, str!("www"), None),
            (2, str!("empty"), None),
            (3, str!("mytest"), None),
            (
                4,
                str!("wanderers-library"),
                Some(str!("wandererslibrary.com")),
            ),
            (5, str!("scp-wiki"), Some(str!("scpwiki.com"))),
        ],
        domains: hashmap! {
            1 => SiteDomainData::default(),
            2 => SiteDomainData::default(),
            3 => SiteDomainData {
                aliases: vec![str!("check")],
                custom_domains: vec![domain!("example.com"), domain!("example.net", false)],
            },
            4 => SiteDomainData {
                aliases: vec![],
                custom_domains: vec![domain!("wandererslibrary.com")],
            },
            5 => SiteDomainData {
                aliases: vec![str!("scpwiki")],
                custom_domains: vec![
                    domain!("scpwiki.com"),
                    domain!("scp-wiki.net", false),
                    domain!("foundation.scp", false),
                    domain!("scp.foundation"),
                ],
            },
        },
    };

    (basic, full)
}

macro_rules! test_file_path {
    ($suffix:expr) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/caddy/Caddyfile.",
            $suffix,
        )
    };
}

const CADDYFILE_BASIC_PROD: &str = test_file_path!("basic_prod");
const CADDYFILE_BASIC_LOCAL: &str = test_file_path!("basic_local");
const CADDYFILE_BASIC_LOCAL_DEV: &str = test_file_path!("basic_localdev");
const CADDYFILE_BASIC_DIFFERENT_PROXIES: &str = test_file_path!("proxies");
const CADDYFILE_FULL_PROD: &str = test_file_path!("full_prod");
const CADDYFILE_LONG_DOMAIN: &str = test_file_path!("long");

/// Overwrite test data, for instance when updating `CaddyService`.
///
/// **This flag will fail the build. Disable it before committing your changes.**
const UPDATE_TEST_FILES: bool = false;

/// Simple implementation to remove trailing newlines.
///
/// We strip off trailing newlines since it's not something
/// we care about, and precisely managing them in generation
/// is a waste of time.
fn trim_string(s: &mut String) {
    while s.ends_with('\n') {
        s.pop();
    }
}

/// Returns the body of a named Caddy snippet, including any nested blocks.
fn snippet_body<'a>(caddyfile: &'a str, name: &str) -> &'a str {
    let marker = format!("({name}) {{");
    let marker_start = caddyfile
        .find(&marker)
        .unwrap_or_else(|| panic!("Caddy snippet {name:?} was not generated"));
    let body = &caddyfile[marker_start + marker.len()..];
    let mut depth = 1;

    for (offset, character) in body.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &body[..offset];
                }
            }
            _ => {}
        }
    }

    panic!("Caddy snippet {name:?} was not closed");
}

#[test]
fn single_upstream_proxies_do_not_eject_their_only_backend() {
    const FRAMERAIL_HOST: &str = "framerail:3393";
    const WWS_HOST: &str = "wws:3466";

    let config = build_config("wikijump.test", "wjfiles.test");
    let (sites, _) = build_site_data();
    let caddyfile = CaddyService::generate_with_data(
        &config,
        &CaddyfileOptions {
            debug: false,
            local: false,
            http_port: None,
            https_port: None,
            wildcard_cert: None,
            deploy_host: None,
            framerail_host: cow!(FRAMERAIL_HOST),
            wws_host: cow!(WWS_HOST),
        },
        &sites,
    )
    .expect("failed to generate Caddyfile");

    for (snippet_name, expected_upstream) in [
        ("reverse_proxy_framerail", FRAMERAIL_HOST),
        ("reverse_proxy_wws", WWS_HOST),
    ] {
        let snippet = snippet_body(&caddyfile, snippet_name);
        let upstreams = snippet
            .lines()
            .map(str::trim)
            .find_map(|line| line.strip_prefix("to "))
            .unwrap_or_else(|| panic!("Caddy snippet {snippet_name:?} has no upstream"))
            .split_whitespace()
            .collect::<Vec<_>>();

        assert_eq!(
            upstreams,
            [expected_upstream],
            "Caddy snippet {snippet_name:?} must keep exactly one direct upstream",
        );

        for line in snippet.lines() {
            let Some(directive) = line.split_whitespace().next() else {
                continue;
            };
            let is_health_directive = directive.starts_with("health_")
                || matches!(
                    directive,
                    "fail_duration"
                        | "max_fails"
                        | "unhealthy_status"
                        | "unhealthy_latency"
                        | "unhealthy_request_count"
                );

            assert!(
                !is_health_directive,
                "single-upstream Caddy snippet {snippet_name:?} must not contain health-ejection directive {directive:?}",
            );
        }
    }
}

/// Reads a `Caddyfile`, which represents the expected output of generation with certain settings.
fn read_test_file(path: &str) -> String {
    let mut file = File::open(path).expect("Unable to open test file");
    let mut caddyfile = String::new();
    file.read_to_string(&mut caddyfile)
        .expect("Unable to read test file");

    trim_string(&mut caddyfile);
    caddyfile
}

/// Writes a `Caddyfile`, in the event that we are updating test files.
/// For convenience when we have updated the `Caddyfile` template in `CaddyService`.
fn write_test_file(path: &str, caddyfile: &str) {
    let mut file = File::create(path).expect("Unable to open test file");
    file.write_all(caddyfile.as_bytes())
        .expect("Unable to write to test file");
    file.write_all(b"\n")
        .expect("Unable to write final newline");
}

#[test]
fn deploy_proxies_strip_internal_headers_in_generated_and_provisional_configs() {
    let configs = [
        (
            "basic production fixture",
            read_test_file(CADDYFILE_BASIC_PROD),
        ),
        (
            "full production fixture",
            read_test_file(CADDYFILE_FULL_PROD),
        ),
        (
            "proxy fixture",
            read_test_file(CADDYFILE_BASIC_DIFFERENT_PROXIES),
        ),
        (
            "development provisional Caddyfile",
            String::from(include_str!("../../../../install/dev/caddy/Caddyfile")),
        ),
        (
            "production provisional Caddyfile",
            String::from(include_str!("../../../../install/prod/caddy/Caddyfile")),
        ),
    ];

    for (name, caddyfile) in configs {
        let deploy_block = caddyfile
            .split_once("deploy.")
            .unwrap_or_else(|| panic!("{name} has no deploy host block"))
            .1
            .split_once('}')
            .unwrap_or_else(|| panic!("{name} has an unterminated deploy host block"))
            .0;

        assert!(
            deploy_block.contains("import strip_headers")
                || deploy_block.contains("request_header -X-Wikijump-*"),
            "{name} does not strip client-supplied Wikijump headers before proxying",
        );
        assert!(
            deploy_block.contains("reverse_proxy"),
            "{name} canary did not select the deploy proxy block",
        );
    }
}

#[test]
fn generate_caddyfiles() {
    const DEPLOY_HOST: &str = "localhost:9120";
    const FRAMERAIL_HOST: &str = "framerail:3393";
    const WWS_HOST: &str = "wws:3466";
    const DNS_WILDCARD: &str = "digitalocean token123";

    // Build different configurations for various test cases
    let config_basic = build_config("wikijump.test", "wjfiles.test");
    let config_local = build_config("wikijump.localhost", "wjfiles.localhost");
    let config_long = build_config(
        "site.wikijump.com",
        "wjfiles.host.site.somedomain.example.com",
    );
    let (sites_basic, sites_full) = build_site_data();

    macro_rules! check {
        ($path:expr, $config:expr, $sites:expr, $options:expr $(,)?) => {{
            let expected = read_test_file($path);
            let actual = {
                let mut caddyfile =
                    CaddyService::generate_with_data(&$config, &$options, &$sites)
                        .expect("failed to generate Caddyfile");

                trim_string(&mut caddyfile);
                caddyfile
            };

            if UPDATE_TEST_FILES {
                write_test_file($path, &actual);
            } else {
                assert_eq!(
                    expected,
                    actual,
                    "\
Generated Caddy file did not match!


UNIT TEST INFO:
* Expected output: {}
* Main domain: {}
* Files domain: {}
* Site data: {}
* Options: {:#?}
",
                    expected,
                    $config.main_domain_no_dot,
                    $config.files_domain_no_dot,
                    stringify!($sites),
                    $options,
                );
            }
        }};
    }

    check!(
        CADDYFILE_BASIC_PROD,
        config_basic,
        sites_basic,
        CaddyfileOptions {
            debug: false,
            local: false,
            http_port: None,
            https_port: None,
            wildcard_cert: Some(cow!(DNS_WILDCARD)),
            deploy_host: Some(cow!(DEPLOY_HOST)),
            framerail_host: cow!(FRAMERAIL_HOST),
            wws_host: cow!(WWS_HOST),
        },
    );

    check!(
        CADDYFILE_BASIC_LOCAL,
        config_local,
        sites_basic,
        CaddyfileOptions {
            debug: false,
            local: true,
            http_port: None,
            https_port: None,
            wildcard_cert: None,
            deploy_host: None,
            framerail_host: cow!(FRAMERAIL_HOST),
            wws_host: cow!(WWS_HOST),
        },
    );

    check!(
        CADDYFILE_BASIC_LOCAL_DEV,
        config_local,
        sites_basic,
        CaddyfileOptions {
            debug: true,
            local: true,
            http_port: Some(8000),
            https_port: Some(8443),
            wildcard_cert: None,
            deploy_host: None,
            framerail_host: cow!(FRAMERAIL_HOST),
            wws_host: cow!(WWS_HOST),
        },
    );

    check!(
        CADDYFILE_BASIC_DIFFERENT_PROXIES,
        config_basic,
        sites_basic,
        CaddyfileOptions {
            debug: false,
            local: false,
            http_port: None,
            https_port: None,
            wildcard_cert: None,
            deploy_host: Some(cow!("komodo_host")),
            framerail_host: cow!("web_proxy_host"),
            wws_host: cow!("wws_proxy_host"),
        },
    );

    check!(
        CADDYFILE_FULL_PROD,
        config_basic,
        sites_full,
        CaddyfileOptions {
            debug: false,
            local: false,
            http_port: None,
            https_port: None,
            wildcard_cert: Some(cow!(DNS_WILDCARD)),
            deploy_host: Some(cow!(DEPLOY_HOST)),
            framerail_host: cow!(FRAMERAIL_HOST),
            wws_host: cow!(WWS_HOST),
        },
    );

    check!(
        CADDYFILE_LONG_DOMAIN,
        config_long,
        sites_basic,
        CaddyfileOptions {
            debug: false,
            local: false,
            http_port: None,
            https_port: None,
            wildcard_cert: None,
            deploy_host: None,
            framerail_host: cow!(FRAMERAIL_HOST),
            wws_host: cow!(WWS_HOST),
        },
    );

    if UPDATE_TEST_FILES {
        // We cannot allow tests to silently pass because someone is just auto-updating
        // whatever is generated to match.
        panic!(
            "UPDATE_TEST_FILES is set to true! Disable this flag before attempting to run CI"
        );
    }
}

#[test]
fn caddyfile_options_debug_redacts_wildcard_cert() {
    const DNS_WILDCARD_CANARY: &str = "provider WILDCARD_DEBUG_CANARY_12345";

    let options = CaddyfileOptions {
        debug: false,
        local: false,
        http_port: None,
        https_port: None,
        wildcard_cert: Some(cow!(DNS_WILDCARD_CANARY)),
        deploy_host: Some(cow!("localhost:9120")),
        framerail_host: cow!("framerail:3393"),
        wws_host: cow!("wws:3466"),
    };

    let debug = format!("{options:#?}");

    assert!(!debug.contains(DNS_WILDCARD_CANARY));
    assert!(!debug.contains("WILDCARD_DEBUG_CANARY_12345"));
    assert!(debug.contains("wildcard_cert"));
    assert!(debug.contains("[redacted]"));
}
