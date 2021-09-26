/*
 * render/html/element/math.rs
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

use super::prelude::*;
use latex2mathml::{latex_to_mathml, DisplayStyle};
use std::num::NonZeroUsize;

pub fn render_math_block(
    log: &Logger,
    ctx: &mut HtmlContext,
    name: Option<&str>,
    latex_source: &str,
) {
    info!(
        log,
        "Rendering math block";
        "name" => name.unwrap_or("<none>"),
        "latex-source" => latex_source,
    );

    let index = ctx.next_equation_index();

    render_latex(
        log,
        ctx,
        name,
        Some(index),
        latex_source,
        DisplayStyle::Block,
    );
}

pub fn render_math_inline(log: &Logger, ctx: &mut HtmlContext, latex_source: &str) {
    info!(
        log,
        "Rendering math inline";
        "latex-source" => latex_source,
    );

    render_latex(log, ctx, None, None, latex_source, DisplayStyle::Inline);
}

fn render_latex(
    log: &Logger,
    ctx: &mut HtmlContext,
    name: Option<&str>,
    index: Option<NonZeroUsize>,
    latex_source: &str,
    display: DisplayStyle,
) {
    let (html_tag, wj_type) = match display {
        DisplayStyle::Block => ("div", "wj-math-block"),
        DisplayStyle::Inline => ("span", "wj-math-inline"),
    };

    // Outer container
    ctx.html()
        .tag(html_tag)
        .attr(attr!(
            "is" => wj_type,
            "class" => "wj-math " wj_type,
            "data-name" => name.unwrap_or(""); if name.is_some(),
        ))
        .contents(|ctx| {
            // Add equation index
            if let Some(index) = index {
                ctx.html()
                    .span()
                    .attr(attr!(
                        "class" => "wj-equation-number",
                    ))
                    .contents(|ctx| {
                        str_write!(ctx, "{}", index);

                        // Add period
                        ctx.html()
                            .span()
                            .attr(attr!(
                                "class" => "wj-equation-sep",
                            ))
                            .inner(log, ".");
                    });
            }

            // Add LaTeX source (hidden)
            ctx.html()
                .pre()
                .attr(attr!(
                    "is" => "wj-math-source",
                    "class" => "wj-math-source wj-hidden",
                    "aria-hidden" => "true",
                ))
                .contents(|ctx| {
                    ctx.html().code().inner(log, latex_source);
                });

            // Add generated MathML
            match latex_to_mathml(latex_source, display) {
                Ok(mathml) => {
                    info!(
                        log,
                        "Processed LaTeX -> MathML";
                        "display" => str!(display),
                        "mathml" => &mathml,
                    );

                    // Inject MathML elements
                    ctx.html()
                        .tag(html_tag)
                        .attr(attr!(
                            "is" => "wj-math-ml",
                            "class" => "wj-math-ml",
                        ))
                        .contents(|ctx| ctx.push_raw_str(&mathml));
                }
                Err(error) => {
                    warn!(
                        log,
                        "Error processing LaTeX -> MathML";
                        "display" => str!(display),
                        "error" => str!(error),
                    );

                    let error = str!(error);

                    ctx.html()
                        .span()
                        .attr(attr!(
                            "is" => "wj-math-error",
                            "class" => "wj-math-error",
                        ))
                        .inner(log, error);
                }
            }
        });
}

pub fn render_equation_reference(log: &Logger, ctx: &mut HtmlContext, name: &str) {
    info!(
        log,
        "Rendering equation reference";
        "name" => name,
    );

    ctx.html()
        .span()
        .attr(attr!("class" => "wj-equation-ref"))
        .contents(|ctx| {
            // Equation marker that is hoverable
            ctx.html()
                .button()
                .attr(attr!(
                    "is" => "wj-equation-ref-marker",
                    "class" => "wj-equation-ref-marker",
                    "type" => "button",
                    "data-name" => name,
                ))
                .inner(log, name);

            // Tooltip shown on hover.
            ctx.html().span().attr(attr!(
                "class" => "wj-equation-ref-tooltip",
                "aria-hidden" => "true",
            ));
            // TODO tooltip contents
        });
}
