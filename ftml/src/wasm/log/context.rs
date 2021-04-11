/*
 * wasm/log/context.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use std::collections::HashMap;
use std::fmt;
use wasm_bindgen::JsValue;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum JsonValue {
    String(String),
    Char(char),
    Number(f64),
    Boolean(bool),
    Null(()),
}

pub type Context<'a> = HashMap<&'a str, JsonValue>;

#[derive(Debug, Default)]
pub struct ContextSerializer<'a>(Context<'a>);

impl<'a> slog::Serializer for ContextSerializer<'a> {
    fn emit_arguments(&mut self, key: slog::Key, value: &fmt::Arguments) -> slog::Result {
        let value = value.to_string();
        let value_json = JsonValue::String(value);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_usize(&mut self, key: slog::Key, value: usize) -> slog::Result {
        let value_json = JsonValue::Number(value as f64);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_isize(&mut self, key: slog::Key, value: isize) -> slog::Result {
        let value_json = JsonValue::Number(value as f64);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_bool(&mut self, key: slog::Key, value: bool) -> slog::Result {
        let value_json = JsonValue::Boolean(value);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_char(&mut self, key: slog::Key, value: char) -> slog::Result {
        let value_json = JsonValue::Char(value);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_u8(&mut self, key: slog::Key, value: u8) -> slog::Result {
        let value = f64::from(value);
        let value_json = JsonValue::Number(value);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_i8(&mut self, key: slog::Key, value: i8) -> slog::Result {
        let value = f64::from(value);
        let value_json = JsonValue::Number(value);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_u16(&mut self, key: slog::Key, value: u16) -> slog::Result {
        let value = f64::from(value);
        let value_json = JsonValue::Number(value);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_i16(&mut self, key: slog::Key, value: i16) -> slog::Result {
        let value = f64::from(value);
        let value_json = JsonValue::Number(value);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_u32(&mut self, key: slog::Key, value: u32) -> slog::Result {
        let value = f64::from(value);
        let value_json = JsonValue::Number(value);
        self.0.insert(key, value_json);
        Ok(())
    }

    fn emit_i32(&mut self, key: slog::Key, value: i32) -> slog::Result {
        let value = f64::from(value);
        let value_json = JsonValue::Number(value);
        self.0.insert(key, value_json);
        Ok(())
    }
}

impl<'a> From<ContextSerializer<'a>> for HashMap<&'a str, JsonValue> {
    #[inline]
    fn from(serializer: ContextSerializer<'a>) -> HashMap<&'a str, JsonValue> {
        serializer.0
    }
}
