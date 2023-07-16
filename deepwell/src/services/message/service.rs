/*
 * services/message/service.rs
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
use cuid2::cuid;

#[derive(Debug)]
pub struct MessageService;

impl MessageService {
    pub async fn send(
        ctx: &ServiceContext<'_>,
        SendMessage {
            sender_id,
            recipients:
                MessageRecipients {
                    regular: recipients,
                    carbon_copy,
                    blind_carbon_copy,
                },
            wikitext,
            reply_to,
        }: SendMessage,
    ) -> Result<()> {
        tide::log::info!("Preparing to send direct message from {sender_id}");

        // TODO validate input, fail if any are not ok
        // - not too many recipients
        // - wikitext isn't too long
        // - not blocked by anyone in the recipient list

        todo!()
    }

    pub async fn get_message_optional(
        ctx: &ServiceContext<'_>,
        record_id: &str,
    ) -> Result<Option<()>> {
        // XXX
        todo!()
    }

    pub async fn get_record_optional(
        ctx: &ServiceContext<'_>,
        record_id: &str,
    ) -> Result<Option<()>> {
        // XXX
        todo!()
    }
}
