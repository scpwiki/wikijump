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
    // Validates an email through the MailCheck API. 
    pub async fn validate(
        ctx: ServiceContext<'_>,
        email: String,
    ) -> Result</*EmailValidationOutput*/ ()> {
        // Sends a GET request to the MailCheck API and deserializes the response.
        let mailcheck_response = surf::get(format!("https://api.mailcheck.ai/email/{email}"))
            .send()
            .await?
            .body_json::<MailCheckResponse>()
            .await?;

        Ok(())
    }
}