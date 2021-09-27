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
use cfg_if::cfg_if;
use std::num::NonZeroUsize;

cfg_if! {
    if #[cfg(feature = "mathml")] {
        use latex2mathml::{latex_to_mathml, DisplayStyle};
    } else {
        /// Mocked version of the enum from `latex2mathml`.
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        enum DisplayStyle {
            Block,
            Inline,
        }
    }
}

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
    // error_type is unused if MathML is disabled
    let (html_tag, wj_type, _error_type) = match display {
        DisplayStyle::Block => ("div", "wj-math-block", "wj-error-block"),
        DisplayStyle::Inline => ("span", "wj-math-inline", "wj-error-inline"),
    };

    // Outer container
    ctx.html()
        .tag(html_tag)
        .attr(attr!(
            "class" => "wj-math " wj_type,
            "data-name" => name.unwrap_or(""); if name.is_some(),
        ))
        .contents(|ctx| {
            // Add equation index
            if let Some(index) = index {
                ctx.html()
                    .span()
                    .attr(attr!("class" => "wj-equation-number"))
                    .contents(|ctx| {
                        // Open parenthesis
                        ctx.html()
                            .span()
                            .attr(attr!(
                                "class" => "wj-equation-paren wj-equation-paren-open",
                            ))
                            .inner(log, "(");

                        str_write!(ctx, "{}", index);

                        // Close parenthesis
                        ctx.html()
                            .span()
                            .attr(attr!(
                                "class" => "wj-equation-paren wj-equation-paren-close",
                            ))
                            .inner(log, ")");
                    });
            }

            // Add LaTeX source (hidden)
            // Can't use a pre tag because that won't work for inline tags
            ctx.html()
                .code()
                .attr(attr!(
                    "class" => "wj-math-source wj-hidden",
                    "aria-hidden" => "true",
                ))
                .inner(log, latex_source);

            // Add generated MathML
            cfg_if! {
                if #[cfg(feature = "mathml")] {
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
                                .span()
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
                                .attr(attr!("class" => _error_type))
                                .inner(log, error);
                        }
                    }
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
