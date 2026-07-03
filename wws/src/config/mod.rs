/*
 * config/mod.rs
 *
 * Wilson's Web Server - Serves a zoo of user-generated content
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

mod args;
mod object;
mod secrets;

pub use self::object::Config;
pub use self::secrets::Secrets;

use self::args::Arguments;
use dotenvy::dotenv;
use ref_map::*;
use s3::creds::Credentials;
use s3::region::Region;
use std::ffi::OsString;
use std::path::PathBuf;
use std::{env, process};

pub fn load_config() -> (Config, Secrets) {
    dotenv().ok();

    match load_config_from(
        Arguments::parse(),
        |name| env::var(name),
        |name| env::var_os(name),
        || {
            Credentials::from_env_specific(
                Some("S3_ACCESS_KEY_ID"),
                Some("S3_SECRET_ACCESS_KEY"),
                None,
                None,
            )
            .map_err(|error| error.to_string())
        },
        |profile_name| {
            Credentials::from_profile(profile_name).map_err(|error| error.to_string())
        },
    ) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

fn load_config_from<GetEnv, GetEnvOs, EnvCredentials, ProfileCredentials>(
    arguments: Arguments,
    get_env: GetEnv,
    get_env_os: GetEnvOs,
    env_credentials: EnvCredentials,
    profile_credentials: ProfileCredentials,
) -> Result<(Config, Secrets), String>
where
    GetEnv: Fn(&str) -> std::result::Result<String, env::VarError>,
    GetEnvOs: Fn(&str) -> Option<OsString>,
    EnvCredentials: Fn() -> Result<Credentials, String>,
    ProfileCredentials: Fn(Option<&str>) -> Result<Credentials, String>,
{
    let required_env = |name| {
        get_env(name).map_err(|error| {
            format!("Unable to read environment variable {name}: {error}")
        })
    };

    // Essentially .expect(), but allows printing the environment variable name in the message.
    macro_rules! required_env {
        ($name:expr) => {
            required_env($name)?
        };
    }

    // Process arguments and overrides
    let Arguments {
        enable_trace,
        enable_deepwell_check,
        mut pid_file,
        mut address,
    } = arguments;

    if let Some(value) = get_env_os("PID_FILE") {
        pid_file = Some(PathBuf::from(value));
    }

    if let Ok(value) = get_env("ADDRESS") {
        address = value
            .parse()
            .map_err(|_| "Unable to parse socket address".to_string())?;
    }

    // Process secrets
    let deepwell_url = required_env!("DEEPWELL_URL");
    let redis_url = required_env!("REDIS_URL");

    let s3_files_bucket = required_env!("S3_FILES_BUCKET");
    let s3_tblocks_bucket = required_env!("S3_TEXT_BLOCKS_BUCKET");

    let s3_region = match get_env("S3_AWS_REGION") {
        // Standard AWS S3 region, parse out into enum.
        Ok(value) => match value.parse() {
            Ok(region) => region,
            Err(error) => {
                return Err(format!(
                    "S3_AWS_REGION variable is not a valid AWS region ID: {error}",
                ));
            }
        },

        // Custom region, with a specific S3 endpoint.
        Err(_) => {
            let region = required_env!("S3_REGION_NAME");
            let endpoint = required_env!("S3_CUSTOM_ENDPOINT");

            Region::Custom { region, endpoint }
        }
    };

    let s3_path_style = match required_env!("S3_PATH_STYLE").parse() {
        Ok(path_style) => path_style,
        Err(_) => {
            return Err("S3_PATH_STYLE variable is not a valid boolean".to_string());
        }
    };

    let s3_credentials = {
        // Try to read from environment
        // Reads from S3_ACCESS_KEY_ID and S3_SECRET_ACCESS_KEY
        let env_creds = env_credentials();

        match env_creds {
            Ok(credentials) => credentials,
            Err(_) => {
                // Try to read from profile
                let profile_name = get_env("AWS_PROFILE_NAME").ok();
                let profile_name = profile_name.ref_map(|s| s.as_str());

                match profile_credentials(profile_name) {
                    Ok(credentials) => credentials,
                    Err(error) => {
                        return Err(format!(
                            "Unable to read AWS credentials file: {error}"
                        ));
                    }
                }
            }
        }
    };

    // Build and return
    let config = Config {
        enable_trace,
        enable_deepwell_check,
        pid_file,
        address,
    };

    let secrets = Secrets {
        deepwell_url,
        redis_url,
        s3_files_bucket,
        s3_tblocks_bucket,
        s3_region,
        s3_path_style,
        s3_credentials,
    };

    Ok((config, secrets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    fn env_reader(
        vars: &'static [(&'static str, &'static str)],
    ) -> impl Fn(&str) -> std::result::Result<String, env::VarError> {
        move |name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_string())
                .ok_or(env::VarError::NotPresent)
        }
    }

    fn env_os_reader(
        vars: &'static [(&'static str, &'static str)],
    ) -> impl Fn(&str) -> Option<OsString> {
        move |name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        }
    }

    fn credentials() -> Credentials {
        Credentials::new(Some("access-key"), Some("secret-key"), None, None, None)
            .unwrap()
    }

    fn arguments() -> Arguments {
        Arguments {
            enable_trace: false,
            enable_deepwell_check: false,
            pid_file: None,
            address: "[::]:3466".parse().unwrap(),
        }
    }

    #[test]
    fn load_config_from_env_builds_custom_region_and_applies_overrides() {
        let vars = &[
            ("PID_FILE", "/tmp/wws.pid"),
            ("ADDRESS", "127.0.0.1:8080"),
            ("DEEPWELL_URL", "http://deepwell:2747"),
            ("REDIS_URL", "redis://cache:6379"),
            ("S3_FILES_BUCKET", "files"),
            ("S3_TEXT_BLOCKS_BUCKET", "text-blocks"),
            ("S3_REGION_NAME", "local"),
            ("S3_CUSTOM_ENDPOINT", "http://minio:9000"),
            ("S3_PATH_STYLE", "true"),
        ];

        let (config, secrets) = load_config_from(
            arguments(),
            env_reader(vars),
            env_os_reader(vars),
            || Ok(credentials()),
            |_| Err("profile should not be used".to_string()),
        )
        .unwrap();

        assert!(!config.enable_trace);
        assert!(!config.enable_deepwell_check);
        assert_eq!(config.pid_file, Some(PathBuf::from("/tmp/wws.pid")));
        assert_eq!(config.address.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(config.address.port(), 8080);
        assert_eq!(secrets.deepwell_url, "http://deepwell:2747");
        assert_eq!(secrets.redis_url, "redis://cache:6379");
        assert_eq!(secrets.s3_files_bucket, "files");
        assert_eq!(secrets.s3_tblocks_bucket, "text-blocks");
        assert!(secrets.s3_path_style);
        assert!(matches!(
            secrets.s3_region,
            Region::Custom { ref region, ref endpoint }
                if region == "local" && endpoint == "http://minio:9000"
        ));
        assert_eq!(
            secrets.s3_credentials.access_key.as_deref(),
            Some("access-key")
        );
    }

    #[test]
    fn load_config_from_env_uses_aws_region_and_profile_credentials_fallback() {
        let vars = &[
            ("DEEPWELL_URL", "http://deepwell:2747"),
            ("REDIS_URL", "redis://cache:6379"),
            ("S3_FILES_BUCKET", "files"),
            ("S3_TEXT_BLOCKS_BUCKET", "text-blocks"),
            ("S3_AWS_REGION", "us-east-1"),
            ("S3_PATH_STYLE", "false"),
            ("AWS_PROFILE_NAME", "wws-test"),
        ];

        let (config, secrets) = load_config_from(
            arguments(),
            env_reader(vars),
            env_os_reader(vars),
            || Err("missing env credentials".to_string()),
            |profile| {
                assert_eq!(profile, Some("wws-test"));
                Ok(credentials())
            },
        )
        .unwrap();

        assert_eq!(config.address.ip(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
        assert_eq!(config.address.port(), 3466);
        assert!(matches!(secrets.s3_region, Region::UsEast1));
        assert!(!secrets.s3_path_style);
    }

    #[test]
    fn load_config_from_env_reports_missing_required_env() {
        let error = load_config_from(
            arguments(),
            env_reader(&[]),
            env_os_reader(&[]),
            || Ok(credentials()),
            |_| Err("profile should not be used".to_string()),
        )
        .unwrap_err();

        assert!(error.contains("Unable to read environment variable DEEPWELL_URL"));
    }

    #[test]
    fn load_config_from_env_reports_invalid_path_style() {
        let vars = &[
            ("DEEPWELL_URL", "http://deepwell:2747"),
            ("REDIS_URL", "redis://cache:6379"),
            ("S3_FILES_BUCKET", "files"),
            ("S3_TEXT_BLOCKS_BUCKET", "text-blocks"),
            ("S3_REGION_NAME", "local"),
            ("S3_CUSTOM_ENDPOINT", "http://minio:9000"),
            ("S3_PATH_STYLE", "sometimes"),
        ];

        let error = load_config_from(
            arguments(),
            env_reader(vars),
            env_os_reader(vars),
            || Ok(credentials()),
            |_| Err("profile should not be used".to_string()),
        )
        .unwrap_err();

        assert_eq!(error, "S3_PATH_STYLE variable is not a valid boolean");
    }
}
