/*
 * config/secrets.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

use dotenvy::dotenv;
use ref_map::*;
use s3::{creds::Credentials, region::Region};
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
    /// Set using environment variable `S3_BUCKET`.
    pub s3_bucket: String,

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
}

impl Secrets {
    pub fn load() -> Self {
        dotenv().ok();

        // Essentially .expect(), but allows inserting the environment variable name.
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

        let s3_bucket = get_env!("S3_BUCKET");
        let s3_region = match env::var("S3_AWS_REGION") {
            // Standard AWS S3 region, parse out into enum.
            Ok(value) => {
                match value.parse() {
                    Ok(region) => region,
                    Err(error) => {
                        eprintln!("S3_AWS_REGION variable is not a valid AWS region ID: {error}");
                        process::exit(1);
                    }
                }
            }

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

        // Build and return
        Secrets {
            database_url,
            redis_url,
            s3_bucket,
            s3_region,
            s3_path_style,
            s3_credentials,
        }
    }
}
