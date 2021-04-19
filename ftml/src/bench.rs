/*
 * bench.rs
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

//! Transient file to occassionally utilize to benchmark the library's performance.
//!
//! On a separate branch because this Rust feature requires nightly.

#[macro_use]
extern crate bencher;
extern crate ftml;

#[macro_use]
extern crate lazy_static;
extern crate slog;

use bencher::Bencher;
use std::fs::File;
use std::io::prelude::*;
use ftml::render::html::HtmlRender;
use ftml::render::Render;

macro_rules! build_logger {
    () => {
        slog::Logger::root(slog::Discard, slog::o!())
    };
}

lazy_static! {
    static ref INPUT: String = {
        let mut contents = String::new();
        let mut file = File::open("scp-1730.ftml").expect("Unable to open file");
        file.read_to_string(&mut contents).expect("Unable to read file");
        contents
    };
}

fn full(bench: &mut Bencher) {
    let log = build_logger!();
    let page_info = PageInfo::dummy();

    bench.iter(|| {
        let mut text = INPUT.clone();

        // Run preprocessor
        ftml::preprocess(&log, &mut text);

        // Run lexer
        let tokens = ftml::tokenize(&log, &text);

        // Run parser
        let result = ftml::parse(&log, &tokens);
        let (tree, _warnings) = result.into();

        // Run HTML renderer
        let _html = HtmlRender.render(&log, &page_info, &tree);
    });
}

fn preprocess(bench: &mut Bencher) {
    let log = build_logger!();

    bench.iter(|| {
        let mut text = INPUT.clone();

        // Run preprocessor
        ftml::preprocess(&log, &mut text);
    });
}

fn tokenize(bench: &mut Bencher) {
    let log = build_logger!();

    let mut text = INPUT.clone();

    // Run preprocessor
    ftml::preprocess(&log, &mut text);

    bench.iter(|| {
        // Run lexer
        let _tokens = ftml::tokenize(&log, &text);
    });
}

fn parse(bench: &mut Bencher) {
    let log = build_logger!();

    let mut text = INPUT.clone();

    // Run preprocessor
    ftml::preprocess(&log, &mut text);

    // Run lexer
    let tokens = ftml::tokenize(&log, &text);

    bench.iter(|| {
        // Run parser
        let result = ftml::parse(&log, &tokens);
        let (_tree, _errors) = result.into();
    });
}

fn render(bench: &mut Bencher) {
    let log = build_logger!();
    let page_info = PageInfo::dummy();

    let mut text = INPUT.clone();

    // Run preprocessor
    ftml::preprocess(&log, &mut text);

    // Run lexer
    let tokens = ftml::tokenize(&log, &text);

    // Run parser
    let result = ftml::parse(&log, &tokens);
    let (tree, _warnings) = result.into();

    bench.iter(|| {
        // Run HTML renderer
        let _html = HtmlRender.render(&log, &page_info, &tree);
    });
}
benchmark_group!(benches, full, preprocess, tokenize, parse, render);
benchmark_main!(benches);
