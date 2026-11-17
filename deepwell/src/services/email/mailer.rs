/*
 * services/email/mailer.rs
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

use missive::providers::{LocalMailer, MailjetMailer};
use missive::{DeliveryResult, Email, EmailClient, MailError, StoredEmail};
use reqwest::Client as ReqwestClient;
use std::fmt::{self, Debug};

/// The backend used to deliver emails.
///
/// Which one is used is determined by the `EMAIL_PROVIDER` environment variable.
/// See `Secrets`.
pub enum EmailMailer {
    /// Stores emails in memory instead of sending them.
    ///
    /// This is the stubbed mailer for local development and tests.
    Local(LocalMailer),

    /// Sends emails for real, using the Mailjet API.
    Mailjet(MailjetMailer),
}

impl EmailMailer {
    pub fn local() -> Self {
        EmailMailer::Local(LocalMailer::new())
    }

    pub fn mailjet(api_key: String, secret_key: String, client: ReqwestClient) -> Self {
        EmailMailer::Mailjet(MailjetMailer::with_client(api_key, secret_key, client))
    }

    /// The name of the email provider in use, as in `EMAIL_PROVIDER`.
    pub fn provider_name(&self) -> &'static str {
        match self {
            EmailMailer::Local(_) => "local",
            EmailMailer::Mailjet(_) => "mailjet",
        }
    }

    /// Delivers the given email using whichever provider is configured.
    pub async fn deliver(&self, email: Email) -> Result<DeliveryResult, MailError> {
        match self {
            EmailMailer::Local(mailer) => EmailClient::new(mailer).deliver(email).await,
            EmailMailer::Mailjet(mailer) => EmailClient::new(mailer).deliver(email).await,
        }
    }

    /// Gets the emails "sent" so far, if using the local provider.
    ///
    /// Emails are returned in the order they were sent.
    /// If a real provider is in use, then `None` is returned,
    /// since we have no record of what was sent.
    pub fn local_emails(&self) -> Option<Vec<StoredEmail>> {
        match self {
            EmailMailer::Local(mailer) => {
                let mut emails = mailer.emails();
                emails.reverse();
                Some(emails)
            }
            EmailMailer::Mailjet(_) => None,
        }
    }

    /// Makes all future deliveries fail (or stop failing if `None`).
    ///
    /// This is for testing error handling, and so only works on the local provider.
    ///
    /// # Panics
    /// If a real provider is in use, since calling this outside of tests is a bug.
    pub fn set_local_failure(&self, message: Option<&str>) {
        match self {
            EmailMailer::Local(mailer) => match message {
                Some(message) => mailer.set_failure(message),
                None => mailer.clear_failure(),
            },
            EmailMailer::Mailjet(_) => {
                panic!("Cannot simulate delivery failure on a real email provider");
            }
        }
    }
}

// Manual implementation since the mailers don't implement Debug.
// This also avoids printing any API credentials.
impl Debug for EmailMailer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("EmailMailer")
            .field(&self.provider_name())
            .finish()
    }
}
