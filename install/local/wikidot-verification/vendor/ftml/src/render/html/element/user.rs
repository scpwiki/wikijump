/*
 * render/html/element/user.rs
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

pub fn render_user(ctx: &mut HtmlContext, name: &str, show_avatar: bool) {
    debug!("Rendering user block (name '{name}', show-avatar {show_avatar})");

    match ctx.layout() {
        Layout::Wikidot => render_user_wikidot(ctx, name, show_avatar),
        Layout::Wikijump => render_user_wikijump(ctx, name, show_avatar),
    }
}

fn render_user_wikidot(ctx: &mut HtmlContext, name: &str, show_avatar: bool) {
    let handle = ctx.handle();

    match handle.get_user_info(name) {
        Some(user_info) => {
            let printuser_class = if show_avatar {
                "printuser avatarhover"
            } else {
                "printuser"
            };

            let wikidot_onclick = format!(
                "WIKIDOT.page.listeners.userInfo({}); return false;",
                user_info.user_id,
            );

            ctx.html()
                .span()
                .attr(attr!("class" => printuser_class))
                .inner(|ctx| {
                    if show_avatar {
                        // Image is wrapped in its own <a>
                        ctx.html()
                            .a()
                            .attr(attr!(
                                "href" => &user_info.user_profile_url,
                                "onclick" => &wikidot_onclick,
                            ))
                            .inner(|ctx| {
                                ctx.html()
                                    .img()
                                    .attr(attr!(
                                        "class" => "small",
                                        "src" => &user_info.user_avatar_data,
                                        "alt" => name,
                                        "style" => handle.get_karma_style(user_info.user_karma),
                                    ));
                            });
                    }

                    // Now, the username (text) with its <a>
                    ctx.html()
                        .a()
                        .attr(attr!(
                            "href" => &user_info.user_profile_url,
                            "onclick" => &wikidot_onclick,
                        ))
                        .contents(name);
                });
        }
        None => {
            let (message_pre, message_post) = {
                let page_info = ctx.info();
                let language = &page_info.language;
                let message_pre = handle.get_message(language, "user-missing-pre");
                let message_post = handle.get_message(language, "user-missing-post");
                (message_pre, message_post)
            };

            ctx.push_escaped(message_pre);

            ctx.html()
                .span()
                .attr(attr!("class" => "error-inline"))
                .inner(|ctx| {
                    // TODO localization
                    ctx.html().em().contents(name);
                });

            ctx.push_escaped(message_post);
        }
    }
}

fn render_user_wikijump(ctx: &mut HtmlContext, name: &str, show_avatar: bool) {
    ctx.html()
        .span()
        .attr(attr!("class" => "wj-user-info"))
        .inner(|ctx| match ctx.handle().get_user_info(name) {
            Some(info) => {
                trace!(
                    "Got user information (user id {}, name {})",
                    info.user_id,
                    info.user_name.as_ref(),
                );

                ctx.html()
                    .a()
                    .attr(attr!(
                        "class" => "wj-user-info-link",
                        "href" => &info.user_profile_url,
                    ))
                    .inner(|ctx| {
                        if show_avatar {
                            ctx.html()
                                .span()
                                .attr(attr!(
                                    "class" => "wj-karma",
                                    "data-karma" => &info.user_karma.to_string(),
                                ))
                                .inner(|ctx| {
                                    ctx.html().sprite("wj-karma");
                                });

                            ctx.html().img().attr(attr!(
                                "class" => "wj-user-info-avatar",
                                "src" => &info.user_avatar_data,
                            ));
                        }

                        ctx.html()
                            .span()
                            .attr(attr!("class" => "wj-user-info-name"))
                            .contents(&info.user_name);
                    });
            }
            None => {
                trace!("No such user found");

                ctx.html()
                    .span()
                    .attr(attr!("class" => "wj-error-inline"))
                    .inner(|ctx| {
                        if show_avatar {
                            // Karma SVG
                            ctx.html()
                                .span()
                                .attr(attr!(
                                    "class" => "wj-karma",
                                    "data-karma" => "0",
                                ))
                                .inner(|ctx| {
                                    ctx.html().sprite("wj-karma");
                                });

                            ctx.html().img().attr(attr!(
                                "class" => "wj-user-info-avatar",
                                "src" => "/files--static/media/bad-avatar.png",
                            ));
                        }

                        ctx.html()
                            .span()
                            .attr(attr!("class" => "wj-user-info-name"))
                            .contents(name);
                    });
            }
        });
}
