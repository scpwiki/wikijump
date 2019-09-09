/*
 * ftml/transform.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

use ftml::prelude::*;
use ftml::handle::NullHandle;

pub type TransformFn = fn(&mut String, bool) -> Result<String>;

#[inline]
pub fn prefilter_only(text: &mut String, _wrap: bool) -> Result<String> {
    let mut text = text.clone();
    prefilter(&mut text, &NullHandle)?;
    Ok(text)
}

pub fn parse_only(text: &mut String, wrap: bool) -> Result<String> {
    let mut text = text.clone();
    prefilter(&mut text, &NullHandle)?;
    let tree = parse(&mut text)?;
    let result = if wrap {
        format!(
            "<html><body><pre><code>\n{:#?}\n</code></pre></body></html>\n",
            &tree
        )
    } else {
        format!("{:#?}", &tree)
    };

    Ok(result)
}

pub fn full_transform(text: &mut String, wrap: bool) -> Result<String> {
    let handle = NullHandle;
    let renderer = HtmlRender::new(&handle);

    let info = PageInfo {
        title: "SCP-XXXX",
        alt_title: Some("The Monster"),
        header: None,
        subheader: None,
        rating: 1000,
        tags: vec!["scp", "keter", "intangible", "k-class-scenario", "ontokinetic"],
    };

    let mut output = renderer.transform(text, info, &handle)?;

    if wrap {
        let mut buffer = str!("<html><head>");

        for meta in &output.meta {
            meta.render(&mut buffer)?;
        }

        buffer.push_str("</head><style>");
        buffer.push_str(&output.style);
        buffer.push_str("</style></head><body>");
        buffer.push_str(&output.html);
        buffer.push_str("\n</body></html>\n");

        output.html = buffer;
    }

    Ok(output.html)
}
