/*
 * ffi/exports.rs
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

use super::html::ftml_html_output;
use super::page_info::ftml_page_info;
use super::prelude::*;
use super::settings::ftml_wikitext_settings;
use super::text::ftml_text_output;
use crate::parsing::ParseWarning;
use crate::render::html::HtmlRender;
use crate::render::text::TextRender;
use crate::render::Render;

fn render<R: Render>(
    c_text: *const c_char,
    c_page_info: *const ftml_page_info,
    c_settings: *const ftml_wikitext_settings,
    renderer: &R,
) -> (R::Output, Vec<ParseWarning>) {
    let log = &get_logger();

    // Convert data from C to Rust
    let mut text = unsafe { cstr_to_string(c_text) };
    let page_info = unsafe {
        c_page_info
            .as_ref()
            .expect("Passed PageInfo structure from C was null")
            .to_page_info()
    };
    let settings = unsafe {
        c_settings
            .as_ref()
            .expect("Passed WikitextSettings structure from C was null")
            .to_wikitext_settings()
    };

    // TODO includer

    crate::preprocess(log, &mut text);
    let tokens = crate::tokenize(log, &text);
    let (tree, warnings) = crate::parse(log, &tokens, &page_info, &settings).into();
    let output = renderer.render(log, &tree, &page_info, &settings);
    (output, warnings)
}

/// Runs the entire ftml rendering pipeline for HTML.
///
/// The first argument must be a pointer to space where
/// the output can be written to. It does not need to be initialized.
#[no_mangle]
pub extern "C" fn ftml_render_html(
    output: *mut ftml_html_output,
    input: *const c_char,
    page_info: *const ftml_page_info,
    settings: *const ftml_wikitext_settings,
) {
    let (rust_output, rust_warnings) = render(input, page_info, settings, &HtmlRender);
    let c_output = unsafe { &mut *output };
    c_output.write_from(rust_output, &rust_warnings);
}

/// Runs the entire ftml rendering pipeline for text.
///
/// The first argument must be a pointer to space where
/// the output can be written to. It does not need to be initialized.
#[no_mangle]
pub extern "C" fn ftml_render_text(
    output: *mut ftml_text_output,
    input: *const c_char,
    page_info: *const ftml_page_info,
    settings: *const ftml_wikitext_settings,
) {
    let (rust_output, rust_warnings) = render(input, page_info, settings, &TextRender);
    let c_output = unsafe { &mut *output };
    c_output.write_from(rust_output, &rust_warnings);
}
