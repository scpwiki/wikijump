/*
 * utils.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

use regex::Regex;

pub trait InPlaceReplace {
    fn ireplace_all(&mut self, pattern: &str, replace_with: &str);
    fn ireplace_all_regex(&mut self, regex: &Regex, replace_with: &str);
    fn ireplace_once(&mut self, pattern: &str, replace_with: &str);
    fn ireplace_once_regex(&mut self, regex: &Regex, replace_with: &str);
}

impl InPlaceReplace for String {
    fn ireplace_all(&mut self, pattern: &str, replace_with: &str) {
        while let Some(idx) = self.find(pattern) {
            self.replace_range(idx..idx+pattern.len(), replace_with);
        }
    }

    fn ireplace_all_regex(&mut self, regex: &Regex, replace_with: &str) {
        while let Some(mtch) = regex.find(&self) {
            self.replace_range(mtch.start()..mtch.end(), replace_with);
        }
    }

    fn ireplace_once(&mut self, pattern: &str, replace_with: &str) {
        if let Some(idx) = self.find(pattern) {
            self.replace_range(idx..idx+pattern.len(), replace_with);
        }
    }

    fn ireplace_once_regex(&mut self, regex: &Regex, replace_with: &str) {
        if let Some(mtch) = regex.find(&self) {
            self.replace_range(mtch.start()..mtch.end(), replace_with);
        }
    }
}
