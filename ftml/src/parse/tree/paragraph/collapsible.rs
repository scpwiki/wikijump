/*
 * parse/tree/paragraph/collapsible.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019-2020 Ammon Smith
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

#[derive(Debug, Default)]
struct Context<'a> {
    start_open: bool,
    show_text: Option<Cow<'a, str>>,
    hide_text: Option<Cow<'a, str>>,
    id: Option<Cow<'a, str>>,
    class: Option<Cow<'a, str>>,
    style: Option<Cow<'a, str>>,
    show: Option<(bool, bool)>, // (top, bottom)
}

pub fn parse(pair: Pair<Rule>) -> Result<Paragraph> {
    let mut ctx = Context::default();
    let mut paragraphs = Vec::new();

    // Parse arguments
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::collapsible_arg => {
                let capture = ARGUMENT_NAME
                    .captures(pair.as_str())
                    .expect("Regular expression ARGUMENT_NAME didn't match");
                let key = capture!(capture, "name");
                let value = get_first_pair!(pair);

                debug_assert_eq!(value.as_rule(), Rule::string);

                let value = interp_str(value.as_str())?;
                parse_arg(&mut ctx, key, value)?;
            }
            Rule::paragraph => {
                let paragraph = Paragraph::from_pair(pair)?;
                paragraphs.push(paragraph);
            }
            _ => panic!("Invalid rule for collapsible: {:?}", pair.as_rule()),
        }
    }

    let Context {
        start_open,
        show_text,
        hide_text,
        id,
        class,
        style,
        show,
    } = ctx;
    let (show_top, show_bottom) = show.unwrap_or((true, false));

    Ok(Paragraph::Collapsible {
        show_text,
        hide_text,
        id,
        class,
        style,
        start_open,
        show_top,
        show_bottom,
        paragraphs,
    })
}

fn parse_arg<'c, 'p>(ctx: &'c mut Context<'p>, key: &'_ str, value: Cow<'p, str>) -> Result<()> {
    #[derive(Debug, Copy, Clone)]
    enum Argument {
        Show,
        Hide,
        HideLocation,
        Fold,
        Id,
        Class,
        Style,
    }

    const COLLAPSIBLE_ARGUMENTS: [(&str, Argument); 7] = [
        ("show", Argument::Show),
        ("hide", Argument::Hide),
        ("hidelocation", Argument::HideLocation),
        ("folded", Argument::Fold),
        ("id", Argument::Id),
        ("class", Argument::Class),
        ("style", Argument::Style),
    ];

    fn get_argument(key: &str) -> Argument {
        for (name, argument) in &COLLAPSIBLE_ARGUMENTS {
            if key.eq_ignore_ascii_case(name) {
                return *argument;
            }
        }

        panic!("Unknown argument for [[collapsible]]: {}", key);
    }

    match get_argument(key) {
        Argument::Show => ctx.show_text = Some(value),
        Argument::Hide => ctx.hide_text = Some(value),
        Argument::HideLocation => {
            let locations = parse_hide_location(value.as_ref())?;
            ctx.show = Some(locations);
        }
        Argument::Fold => ctx.start_open = parse_fold(value.as_ref())?,
        Argument::Id => ctx.id = Some(value),
        Argument::Class => ctx.class = Some(value),
        Argument::Style => ctx.style = Some(value),
    }

    Ok(())
}

fn parse_hide_location(value: &str) -> Result<(bool, bool)> {
    const HIDE_LOCATION_ARGUMENTS: [(&str, (bool, bool)); 6] = [
        ("top", (true, false)),
        ("bottom", (false, true)),
        ("both", (true, true)),
        ("neither", (false, false)),
        ("none", (false, false)),
        ("hide", (false, false)),
    ];

    for &(name, locations) in &HIDE_LOCATION_ARGUMENTS {
        if name.eq_ignore_ascii_case(value) {
            return Ok(locations);
        }
    }

    Err(Error::Msg(format!(
        "Invalid hideLocation value: '{}' (must be 'top', 'bottom', 'both', 'neither')",
        value,
    )))
}

fn parse_fold(value: &str) -> Result<bool> {
    const FOLD_ARGUMENTS: [(&str, bool); 6] = [
        ("yes", true),
        ("true", true),
        ("show", true),
        ("no", false),
        ("false", false),
        ("hide", false),
    ];

    for &(name, fold) in &FOLD_ARGUMENTS {
        if name.eq_ignore_ascii_case(value) {
            return Ok(fold);
        }
    }

    Err(Error::Msg(format!(
        "Invalid fold value: '{}' (must be 'yes' or 'no')",
        value,
    )))
}
