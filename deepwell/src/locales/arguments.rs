/*
 * locales/arguments.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

use fluent::{FluentArgs, FluentValue};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageArguments<'a> {
    #[serde(flatten)]
    inner: HashMap<Cow<'a, str>, MessageValue<'a>>,
}

impl<'a> MessageArguments<'a> {
    pub fn to_fluent_args(&self) -> FluentArgs {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageValue<'a> {
    String(Cow<'a, str>),
    Float(f64),
    // for now, we're not adding full FluentNumberOptions support here
    // if we need it in the future we can cross that bridge when we get there
    Null,
}

impl<'a> From<MessageValue<'a>> for FluentValue<'a> {
    fn from(value: MessageValue<'a>) -> FluentValue<'a> {
        match value {
            MessageValue::String(value) => FluentValue::String(value),
            MessageValue::Float(value) => FluentValue::Number(value.into()),
            MessageValue::Null => FluentValue::None,
        }
    }
}
