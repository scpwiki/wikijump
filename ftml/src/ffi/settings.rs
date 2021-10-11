/*
 * ffi/settings.rs
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

use crate::settings::{WikitextMode, WikitextSettings};
use std::ptr;

#[repr(C)]
#[derive(Debug)]
pub struct ftml_wikitext_settings {
    pub mode: ftml_wikitext_mode,
    pub enable_page_syntax: bool,
    pub use_random_ids: bool,
    pub allow_local_paths: bool,
}

impl ftml_wikitext_settings {
    pub unsafe fn to_wikitext_settings(&self) -> WikitextSettings {
        WikitextSettings {
            mode: self.mode.into(),
            enable_page_syntax: self.enable_page_syntax,
            use_random_ids: self.use_random_ids,
            allow_local_paths: self.allow_local_paths,
        }
    }
}

impl From<WikitextSettings> for ftml_wikitext_settings {
    fn from(settings: WikitextSettings) -> ftml_wikitext_settings {
        ftml_wikitext_settings {
            mode: settings.mode.into(),
            enable_page_syntax: settings.enable_page_syntax,
            use_random_ids: settings.use_random_ids,
            allow_local_paths: settings.allow_local_paths,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum ftml_wikitext_mode {
    PAGE,
    DRAFT,
    FORUM_POST,
    DIRECT_MESSAGE,
    LIST,
}

impl From<ftml_wikitext_mode> for WikitextMode {
    #[inline]
    fn from(mode: ftml_wikitext_mode) -> WikitextMode {
        use ftml_wikitext_mode::*;

        match mode {
            PAGE => WikitextMode::Page,
            DRAFT => WikitextMode::Draft,
            FORUM_POST => WikitextMode::ForumPost,
            DIRECT_MESSAGE => WikitextMode::DirectMessage,
            LIST => WikitextMode::List,
        }
    }
}

impl From<WikitextMode> for ftml_wikitext_mode {
    #[inline]
    fn from(mode: WikitextMode) -> ftml_wikitext_mode {
        use ftml_wikitext_mode::*;

        match mode {
            WikitextMode::Page => PAGE,
            WikitextMode::Draft => DRAFT,
            WikitextMode::ForumPost => FORUM_POST,
            WikitextMode::DirectMessage => DIRECT_MESSAGE,
            WikitextMode::List => LIST,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ftml_wikitext_settings_default(
    settings: *mut ftml_wikitext_settings,
) {
    // Do nothing if they passed NULL
    if settings.is_null() {
        return;
    }

    // Write to the destination memory without reading or dropping it
    let c_settings = WikitextSettings::default().into();

    ptr::write(settings, c_settings);
}

#[no_mangle]
pub unsafe extern "C" fn ftml_wikitext_settings_set_mode(
    settings: *mut ftml_wikitext_settings,
    mode: ftml_wikitext_mode,
) {
    // Copy data from pointer
    let mut rust_settings = settings
        .as_mut()
        .expect("Passed WikitextSettings struct from C was null")
        .to_wikitext_settings();

    // Set mode
    rust_settings.set_mode(mode.into());

    // Copy data to pointer
    ptr::write(settings, rust_settings.into());
}
