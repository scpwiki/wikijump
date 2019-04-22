/*
 * render/html/word.rs
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

use self::Word::*;
use super::prelude::*;

pub fn render_words<'a, I, W> (buffer: &mut String, words: I) -> Result<()>
where
    I: IntoIterator<Item = W>,
    W: AsRef<Word<'a>>,
{
    for word in words {
        render_word(buffer, word.as_ref())?;
    }

    Ok(())
}

pub fn render_word(buffer: &mut String, word: &Word) -> Result<()> {
    match word {
        &Anchor { name, ref arguments } => {
            buffer.push_str("<a");

            for (key, value) in arguments.iter() {
                write_tag_arg(buffer, key, Some(value))?;
            }

            buffer.push_str("></a>\n");
        },

        _ => panic!("Word case not implemented yet!"),
    }

    Ok(())
}
