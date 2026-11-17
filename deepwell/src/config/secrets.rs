/*
 * config/secrets.rs
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

use crate::utils::trim_spaces_in_place;
use dotenvy::dotenv;
use ref_map::*;
use s3::creds::Credentials;
use s3::region::Region;
use std::env::VarError;
use std::{env, process};

#[derive(Debug, Clone)]
pub struct Secrets {
    /// The URL of the PostgreSQL database to connect to.
    ///
    /// Set using environment variable `DATABASE_URL`.
    pub database_url: String,

    /// The URL of the Redis database to connect to.
    ///
    /// Set using environment variable `REDIS_URL`.
    pub redis_url: String,

    /// The name of the S3 bucket that file blobs are kept in.
    /// The bucket must already exist prior to program invocation.
    ///
    /// Set using environment variable `S3_FILES_BUCKET`.
    pub s3_files_bucket: String,

    /// The name of the S3 bucket that hosted text blocks are kept in.
    /// The bucket must already exist prior to program invocation.
    ///
    /// Set using environment variable `S3_TEXT_BLOCKS_BUCKET`.
    pub s3_tblocks_bucket: String,

    /// The region to use for S3.
    ///
    /// Set using environment variable `S3_AWS_REGION` if standard,
    /// or `S3_REGION_NAME` and `S3_CUSTOM_ENDPOINT` if custom.
    pub s3_region: Region,

    /// Whether to use path style for S3.
    ///
    /// Set using environment variable `S3_PATH_STYLE`.
    pub s3_path_style: bool,

    /// The credentials to use for S3.
    ///
    /// Set using environment variable `S3_ACCESS_KEY_ID` and `S3_SECRET_ACCESS_KEY`.
    ///
    /// Alternatively you can have it read from the AWS credentials file.
    /// The profile to read from can be set in the `AWS_PROFILE_NAME` environment variable.
    pub s3_credentials: Credentials,

    /// MailCheck API key.
    ///
    /// **Optional**. The server will still work without an API key, but MailCheck will
    /// heavily ratelimit you.
    ///
    /// * No API key: 5 requests / hour
    /// * Free tier: 1000 requests / month
    pub mailcheck_api_key: Option<String>,

    /// Which provider to use for sending emails, and its credentials.
    ///
    /// Set using environment variable `EMAIL_PROVIDER`:
    /// * `local`: Emails are not sent, only stored in memory.
    ///   For local development and tests.
    /// * `mailjet`: Emails are sent using the Mailjet API.
    ///   Requires `MAILJET_API_KEY` and `MAILJET_SECRET_KEY` to be set.
    ///
    /// If unset, then `local` is assumed.
    pub email_provider: EmailProvider,
}

/// The provider used for sending emails, along with any needed credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailProvider {
    /// Don't actually send emails, only store them in memory.
    Local,

    /// Send emails through the Mailjet API.
    Mailjet { api_key: String, secret_key: String },
}

impl EmailProvider {
    /// Determines the email provider from its environment variable values.
    ///
    /// The `get_secret` closure is given the name of an environment variable,
    /// and returns its value if it is set.
    pub fn parse<F>(provider: Option<&str>, mut get_secret: F) -> Result<Self, String>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let mut get_required = |provider: &str, name: &str| {
            let mut value = get_secret(name).unwrap_or_default();
            trim_spaces_in_place(&mut value);
            if value.is_empty() {
                return Err(format!(
                    "EMAIL_PROVIDER is '{provider}', but environment variable {name} is not set",
                ));
            }

            Ok(value)
        };

        match provider.map(str::trim) {
            None | Some("") => Ok(EmailProvider::Local),
            Some(value) if value.eq_ignore_ascii_case("local") => {
                Ok(EmailProvider::Local)
            }
            Some(value) if value.eq_ignore_ascii_case("mailjet") => {
                let api_key = get_required("mailjet", "MAILJET_API_KEY")?;
                let secret_key = get_required("mailjet", "MAILJET_SECRET_KEY")?;
                Ok(EmailProvider::Mailjet {
                    api_key,
                    secret_key,
                })
            }
            Some(value) => Err(format!(
                "EMAIL_PROVIDER variable is not a known provider (expected 'local' or 'mailjet'): '{value}'",
            )),
        }
    }
}

impl Secrets {
    pub fn load() -> Self {
        dotenv().ok();

        // Essentially .expect(), but allows printing the environment variable name in the message.
        macro_rules! get_env {
            ($name:expr) => {
                match env::var($name) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!(
                            "Unable to read environment variable {}: {}",
                            $name, error,
                        );
                        process::exit(1);
                    }
                }
            };
        }

        let database_url = get_env!("DATABASE_URL");
        let redis_url = get_env!("REDIS_URL");

        let s3_files_bucket = get_env!("S3_FILES_BUCKET");
        let s3_tblocks_bucket = get_env!("S3_TEXT_BLOCKS_BUCKET");

        let s3_region = match env::var("S3_AWS_REGION") {
            // Standard AWS S3 region, parse out into enum.
            Ok(value) => match value.parse() {
                Ok(region) => region,
                Err(error) => {
                    eprintln!(
                        "S3_AWS_REGION variable is not a valid AWS region ID: {error}"
                    );
                    process::exit(1);
                }
            },

            // Custom region, with a specific S3 endpoint.
            Err(_) => {
                let region = get_env!("S3_REGION_NAME");
                let endpoint = get_env!("S3_CUSTOM_ENDPOINT");

                Region::Custom { region, endpoint }
            }
        };

        let s3_path_style = match get_env!("S3_PATH_STYLE").parse() {
            Ok(path_style) => path_style,
            Err(_) => {
                eprintln!("S3_PATH_STYLE variable is not a valid boolean");
                process::exit(1);
            }
        };

        let s3_credentials = {
            // Try to read from environment
            // Reads from S3_ACCESS_KEY_ID and S3_SECRET_ACCESS_KEY
            let env_creds = Credentials::from_env_specific(
                Some("S3_ACCESS_KEY_ID"),
                Some("S3_SECRET_ACCESS_KEY"),
                None,
                None,
            );

            match env_creds {
                Ok(credentials) => credentials,
                Err(_) => {
                    // Try to read from profile
                    let profile_name = env::var("AWS_PROFILE_NAME").ok();
                    let profile_name = profile_name.ref_map(|s| s.as_str());

                    match Credentials::from_profile(profile_name) {
                        Ok(credentials) => credentials,
                        Err(error) => {
                            eprintln!("Unable to read AWS credentials file: {error}");
                            process::exit(1);
                        }
                    }
                }
            }
        };

        let mailcheck_api_key = match env::var("MAILCHECK_API_KEY") {
            Ok(value) if value.is_empty() => None,
            Ok(value) => Some(value),
            Err(VarError::NotPresent) => None,
            Err(error) => {
                eprintln!(
                    "Unable to read environment variable MAILCHECK_API_KEY: {error}"
                );
                process::exit(1);
            }
        };

        let email_provider = {
            // Unlike a missing variable, an unreadable one (i.e. not valid UTF-8)
            // is an error, rather than being treated as if it were unset.
            let get_optional = |name: &str| match env::var(name) {
                Ok(value) => Some(value),
                Err(VarError::NotPresent) => None,
                Err(error) => {
                    eprintln!("Unable to read environment variable {name}: {error}");
                    process::exit(1);
                }
            };

            let provider = get_optional("EMAIL_PROVIDER");
            let result =
                EmailProvider::parse(provider.ref_map(|s| s.as_str()), get_optional);

            match result {
                Ok(email_provider) => email_provider,
                Err(message) => {
                    eprintln!("{message}");
                    process::exit(1);
                }
            }
        };

        // Build and return
        Secrets {
            database_url,
            redis_url,
            s3_files_bucket,
            s3_tblocks_bucket,
            s3_region,
            s3_path_style,
            s3_credentials,
            mailcheck_api_key,
            email_provider,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(
        provider: Option<&str>,
        secrets: &[(&str, &str)],
    ) -> Result<EmailProvider, String> {
        EmailProvider::parse(provider, |name| {
            secrets
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| str!(*value))
        })
    }

    #[test]
    fn email_provider_local() {
        assert_eq!(parse(None, &[]), Ok(EmailProvider::Local));
        assert_eq!(parse(Some(""), &[]), Ok(EmailProvider::Local));
        assert_eq!(parse(Some("local"), &[]), Ok(EmailProvider::Local));
        assert_eq!(parse(Some("LOCAL"), &[]), Ok(EmailProvider::Local));
    }

    #[test]
    fn email_provider_mailjet() {
        const SECRETS: [(&str, &str); 2] =
            [("MAILJET_API_KEY", "abc"), ("MAILJET_SECRET_KEY", "xyz")];

        assert_eq!(
            parse(Some("mailjet"), &SECRETS),
            Ok(EmailProvider::Mailjet {
                api_key: str!("abc"),
                secret_key: str!("xyz"),
            }),
        );

        // Whitespace around credentials is ignored
        assert_eq!(
            parse(
                Some(" mailjet "),
                &[("MAILJET_API_KEY", " abc"), ("MAILJET_SECRET_KEY", "xyz\n")],
            ),
            Ok(EmailProvider::Mailjet {
                api_key: str!("abc"),
                secret_key: str!("xyz"),
            }),
        );

        // Missing or empty credentials
        parse(
            Some("mailjet"),
            &[("MAILJET_API_KEY", "abc"), ("MAILJET_SECRET_KEY", "  ")],
        )
        .unwrap_err();
        parse(Some("mailjet"), &[]).unwrap_err();
        parse(Some("mailjet"), &SECRETS[..1]).unwrap_err();
        parse(Some("mailjet"), &SECRETS[1..]).unwrap_err();
        parse(
            Some("mailjet"),
            &[("MAILJET_API_KEY", "abc"), ("MAILJET_SECRET_KEY", "")],
        )
        .unwrap_err();
    }

    #[test]
    fn email_provider_unknown() {
        parse(Some("sendgrid"), &[]).unwrap_err();
        parse(Some("mailjet2"), &[]).unwrap_err();
    }
}
