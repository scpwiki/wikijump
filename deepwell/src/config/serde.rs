/*
 * config/serde.rs
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

use tide::log::LevelFilter;

#[derive(Debug, Copy, Clone)]
pub struct LogLevel(LevelFilter);

pub fn parse_log_level(value: &str) -> Option<LevelFilter> {
    const LEVELS: [(&str, LevelFilter); 10] = [
        ("off", LevelFilter::Off),
        ("err", LevelFilter::Error),
        ("error", LevelFilter::Error),
        ("warn", LevelFilter::Warn),
        ("warning", LevelFilter::Warn),
        ("info", LevelFilter::Info),
        ("information", LevelFilter::Info),
        ("debug", LevelFilter::Debug),
        ("trace", LevelFilter::Trace),
        ("all", LevelFilter::Trace),
    ];

    for &(name, level) in &LEVELS {
        if value.eq_ignore_ascii_case(name) {
            return Some(level);
        }
    }

    None
}
