/*
 * examples/dump_json.rs
 *
 * ftml - Library to parse Wikidot text
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

//! Helper script to generate JSON output containing syntax trees.
//!
//! Run via `scripts/dump_json.sh`

extern crate clap;
extern crate ftml;
extern crate serde_json;

use clap::{Arg, ArgAction, Command, value_parser};
use ftml::data::{PageInfo, ScoreValue};
use ftml::layout::Layout;
use ftml::settings::{WikitextMode, WikitextSettings};
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

macro_rules! cow {
    ($value:expr) => {
        Cow::Borrowed($value)
    };
}

// Command-line processing

#[derive(Debug, Copy, Clone)]
enum OutputType {
    Json,
    Rust,
}

#[derive(Debug, Copy, Clone)]
enum OutputField {
    SyntaxTree,
    Errors,
}

#[derive(Debug)]
struct Config {
    output_type: OutputType,
    output_field: OutputField,
    pretty: bool,
    layout: Layout,
    input_path: Option<PathBuf>,
    page_info: Option<PageInfo<'static>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            output_type: OutputType::Json,
            output_field: OutputField::SyntaxTree,
            pretty: true,
            layout: Layout::Wikidot,
            input_path: None,
            page_info: None,
        }
    }
}

fn parse_args() -> Config {
    let mut config = Config::default();
    let mut matches = Command::new("dump_json")
        .author(ftml::info::PKG_AUTHORS)
        .version(ftml::info::VERSION.as_str())
        .long_version(ftml::info::FULL_VERSION.as_str())
        .about(ftml::info::PKG_DESCRIPTION)
        .arg(
            Arg::new("rust-output")
                .short('R')
                .long("rust")
                .action(ArgAction::SetTrue)
                .help("Emit Rust debug output instead of JSON."),
        )
        .arg(
            Arg::new("compact-output")
                .short('k')
                .long("compact")
                .action(ArgAction::SetTrue)
                .help("Emit compact output instead of prettified."),
        )
        .arg(
            Arg::new("layout")
                .short('l')
                .long("layout")
                .action(ArgAction::Set)
                .help("Specify an output layout. (one of 'wikidot', 'wikijump')"),
        )
        .arg(
            Arg::new("page-info")
                .long("page-info")
                .value_name("JSON")
                .action(ArgAction::Set)
                .help("Specify a custom page info object for use."),
        )
        .arg(
            Arg::new("error-output")
                .short('e')
                .long("emit-errors")
                .action(ArgAction::SetTrue)
                .help("Emit the list of errors instead of the syntax tree."),
        )
        .arg(
            Arg::new("input-file")
                .value_parser(value_parser!(PathBuf))
                .value_name("PATH")
                .help("Read wikitext from this file instead of stdin."),
        )
        .get_matches();

    if matches.remove_one::<bool>("rust-output") == Some(true) {
        config.output_type = OutputType::Rust;
    }

    if matches.remove_one::<bool>("error-output") == Some(true) {
        config.output_field = OutputField::Errors;
    }

    if matches.remove_one::<bool>("compact-output") == Some(true) {
        config.pretty = false;
    }

    if let Some(layout) = matches.remove_one::<String>("layout") {
        config.layout = layout.parse().expect("Invalid layout value");
    }

    if let Some(json) = matches.remove_one::<String>("page-info") {
        let page_info =
            serde_json::from_str(&json).expect("Unable to read custom page info JSON");
        config.page_info = Some(page_info);
    }

    config.input_path = matches.remove_one::<PathBuf>("input-file");

    config
}

// Preparation

fn get_wikitext(path: Option<&Path>) -> String {
    let mut buffer = String::new();

    match path {
        Some(path) => {
            let mut file = File::open(path).expect("Unable to open file");
            file.read_to_string(&mut buffer)
                .expect("Unable to read wikitext from file");
        }
        None => {
            let mut stream = io::stdin().lock();
            stream
                .read_to_string(&mut buffer)
                .expect("Unable to read wikitext from stdin");
        }
    }

    buffer
}

fn default_page_info() -> PageInfo<'static> {
    PageInfo {
        page: cow!("page"),
        category: None,
        site: cow!("ast-dump"),
        title: cow!("AST JSON Dump"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: vec![],
        language: cow!("default"),
    }
}

fn get_settings(mut config: Config) -> (PageInfo<'static>, WikitextSettings) {
    let page_info = config.page_info.take().unwrap_or_else(default_page_info);
    let settings = WikitextSettings::from_mode(WikitextMode::Page, config.layout);
    (page_info, settings)
}

// Main functions

fn output_data<T: Serialize + Debug>(
    output_type: OutputType,
    pretty_print: bool,
    data: &T,
) {
    macro_rules! print_json {
        ($method:ident) => {{
            let mut stream = io::stdout().lock();
            serde_json::$method(&mut stream, data).expect("Unable to emit JSON");
        }};
    }

    match (output_type, pretty_print) {
        (OutputType::Json, true) => print_json!(to_writer_pretty),
        (OutputType::Json, false) => print_json!(to_writer),
        (OutputType::Rust, true) => println!("{data:#?}"),
        (OutputType::Rust, false) => println!("{data:?}"),
    }
}

fn main() {
    let config = parse_args();
    let input = get_wikitext(config.input_path.as_deref());
    let (output_type, output_field, pretty_print) =
        (config.output_type, config.output_field, config.pretty);
    let (page_info, parse_settings) = get_settings(config);

    let (mut wikitext, _pages) = ftml::include(
        &input,
        &parse_settings,
        ftml::includes::NullIncluder,
        || unreachable!(),
    )
    .unwrap_or_else(|x| match x {});

    ftml::preprocess(&mut wikitext);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &parse_settings);
    let (tree, errors) = result.into();

    match output_field {
        OutputField::SyntaxTree => output_data(output_type, pretty_print, &tree),
        OutputField::Errors => output_data(output_type, pretty_print, &errors),
    }
}
