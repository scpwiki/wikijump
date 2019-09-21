/*
 * render/html/module/rate.rs
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

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct RateModule;

impl Module for RateModule {
    fn render(
        ctx: &mut HtmlContext,
        arguments: &HashMap<&str, Cow<str>>,
        contents: Option<&str>,
    ) -> Result<()> {
        if !arguments.is_empty() {
            return Err(Error::StaticMsg(
                "No arguments are expected for the rate module",
            ));
        }

        if contents.is_some() {
            return Err(Error::StaticMsg("The rate module should not have contents"));
        }

        let rating = ctx.info().rating;

        ctx.html()
            .div()
            .attr("class", &["rating-module"])
            .contents(|ctx| {
                ctx.push_raw('[');
                ctx.html()
                    .b()
                    .attr("class", &["page-rating"])
                    .contents(|ctx| {
                        write!(ctx, "{:+}", rating)?;
                        Ok(())
                    })?;
                ctx.push_raw(']');

                ctx.html().b().contents(|ctx| {
                    ctx.html()
                        .a()
                        .attr("onclick", &["scpvote(-1);"])
                        .inner(&"-")?;
                    Ok(())
                })?;

                ctx.html().b().contents(|ctx| {
                    ctx.html()
                        .a()
                        .attr("onclick", &["scpvote(0);"])
                        .inner(&"0")?;
                    Ok(())
                })?;

                ctx.html().b().contents(|ctx| {
                    ctx.html()
                        .a()
                        .attr("onclick", &["scpvote(+1);"])
                        .inner(&"+")?;
                    Ok(())
                })?;

                Ok(())
            })?;

        Ok(())
    }
}
