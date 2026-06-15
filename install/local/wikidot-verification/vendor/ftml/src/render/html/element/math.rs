/*
 * render/html/element/math.rs
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

pub fn render_math_block(ctx: &mut HtmlContext, name: Option<&str>, latex_source: &str) {
    debug!(
        "Rendering math block (name '{}', source '{}')",
        name.unwrap_or("<none>"),
        latex_source,
    );

    let index = ctx.next_equation_index();

    render_latex(ctx, name, Some(index), latex_source, DisplayStyle::Block);
}

pub fn render_math_inline(ctx: &mut HtmlContext, latex_source: &str) {
    debug!("Rendering math inline (source '{latex_source}'");
    render_latex(ctx, None, None, latex_source, DisplayStyle::Inline);
}

fn render_latex(
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
        .inner(|ctx| {
            // Add equation index
            if let Some(index) = index {
                ctx.html()
                    .span()
                    .attr(attr!("class" => "wj-equation-number"))
                    .inner(|ctx| {
                        // Open parenthesis
                        ctx.html()
                            .span()
                            .attr(attr!(
                                "class" => "wj-equation-paren wj-equation-paren-open",
                            ))
                            .contents("(");

                        str_write!(ctx, "{index}");

                        // Close parenthesis
                        ctx.html()
                            .span()
                            .attr(attr!(
                                "class" => "wj-equation-paren wj-equation-paren-close",
                            ))
                            .contents(")");
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
                .contents(latex_source);

            // Add generated MathML
            cfg_if! {
                if #[cfg(feature = "mathml")] {
                    match latex_to_mathml(latex_source, display) {
                        Ok(mathml) => {
                            debug!("Processed LaTeX -> MathML");

                            // Inject MathML elements
                            ctx.html()
                                .element("wj-math-ml")
                                .attr(attr!("class" => "wj-math-ml"))
                                .inner(|ctx| ctx.push_raw_str(&mathml));
                        }
                        Err(error) => {
                            warn!("Error processing LaTeX -> MathML: {error}");
                            let error = str!(error);

                            ctx.html()
                                .span()
                                .attr(attr!("class" => _error_type))
                                .contents(error);
                        }
                    }
                }
            }
        });
}

pub fn render_equation_reference(ctx: &mut HtmlContext, name: &str) {
    debug!("Rendering equation reference (name '{name}')");

    ctx.html()
        .span()
        .attr(attr!("class" => "wj-equation-ref"))
        .inner(|ctx| {
            // Equation marker that is hoverable
            ctx.html()
                .element("wj-equation-ref-marker")
                .attr(attr!(
                    "class" => "wj-equation-ref-marker",
                    "type" => "button",
                    "data-name" => name,
                ))
                .contents(name);

            // Tooltip shown on hover.
            ctx.html().span().attr(attr!(
                "class" => "wj-equation-ref-tooltip",
                "aria-hidden" => "true",
            ));
            // TODO tooltip contents
        });
}
