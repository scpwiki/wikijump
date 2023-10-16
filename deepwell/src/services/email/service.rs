/*
 * services/email/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

use super::prelude::*;

#[derive(Debug)]
pub struct EmailService;

impl EmailService {
    /// Validates an email through the MailCheck API.
    pub async fn validate(email: &str) -> Result<EmailValidationOutput> {
        // Sends a GET request to the MailCheck API and deserializes the response.
        let mailcheck = reqwest::get(format!("https://api.mailcheck.ai/email/{email}"))
            .await?
            .json::<MailCheckResponse>()
            .await?;

        // Create the output with default parameters.
        let mut output = EmailValidationOutput::default();

        // Check request status.
        match mailcheck.status {
            // Valid request.
            200 => {}

            // Invalid request.
            400 => {
                tide::log::error!(
                    "MailCheck API request failed with bad response: {:?}",
                    mailcheck.error,
                );
                return Err(Error::EmailVerification(mailcheck.error));
            }

            // Exceeded rate limit.
            429 => {
                tide::log::error!("MailCheck API hit ratelimit: {:?}", mailcheck.error);
                return Err(Error::RateLimited);
            }

            // Other statuses.
            _ => {
                tide::log::warn!(
                    "MailCheck API returned status {}: {:?}",
                    mailcheck.status,
                    mailcheck.error,
                );
            }
        }

        // Check if the email is an alias.
        if mailcheck.alias {
            output.classification = EmailClassification::Alias;
        }

        // Check if the email is a disposable.
        if mailcheck.disposable {
            output.valid = false;
            output.classification = EmailClassification::Disposable;
        }

        // Check if the domain has any MX records.
        if !mailcheck.mx {
            output.valid = false;
            output.classification = EmailClassification::Invalid;
        }

        // Set "did you mean" field to mailcheck response.
        output.did_you_mean = mailcheck.did_you_mean;

        Ok(output)
    }
}
