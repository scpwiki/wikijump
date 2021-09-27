/*
 * render/html/element/user.rs
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

pub fn render_user(log: &Logger, ctx: &mut HtmlContext, name: &str, show_avatar: bool) {
    info!(
        log,
        "Rendering user block";
        "name" => name,
        "show-avatar" => show_avatar,
    );

    ctx.html()
        .span()
        .attr(attr!(
            "is" => "wj-user-info",
            "class" => "wj-user-info",
        ))
        .contents(|ctx| match ctx.handle().get_user_info(log, name) {
            Some(info) => {
                debug!(
                    log,
                    "Got user information";
                    "user-id" => info.user_id,
                    "user-name" => info.user_name.as_ref(),
                );

                ctx.html()
                    .a()
                    .attr(attr!("href" => &info.user_profile_url))
                    .contents(|ctx| {
                        if show_avatar {
                            ctx.html()
                                .span()
                                .attr(attr!(
                                    "class" => "wj-karma",
                                    "data-karma" => &info.user_karma.to_string(),
                                ))
                                .contents(|ctx| {
                                    ctx.html().svg_use(
                                        "/files--common/media/karma.svg#wj-karma-symbol",
                                        "0 0 64 114",
                                    );
                                });

                            ctx.html().img().attr(attr!(
                                "class" => "wj-user-info-avatar",
                                "src" => &info.user_avatar_data,
                            ));
                        }

                        ctx.html()
                            .span()
                            .attr(attr!("class" => "wj-user-info-name"))
                            .inner(log, &info.user_name);
                    });
            }
            None => {
                debug!(log, "No such user found");

                ctx.html()
                    .span()
                    .attr(attr!("class" => "wj-error-inline"))
                    .contents(|ctx| {
                        if show_avatar {
                            // Karma SVG
                            ctx.html()
                                .span()
                                .attr(attr!(
                                    "class" => "wj-karma",
                                    "data-karma" => "0",
                                ))
                                .contents(|ctx| {
                                    ctx.html().svg_use(
                                        "/files--common/media/karma.svg#wj-karma-symbol",
                                        "0 0 64 114",
                                    );
                                });

                            ctx.html().img().attr(attr!(
                                "class" => "wj-user-info-avatar",
                                "src" => "/files--common/media/bad-avatar.png",
                            ));
                        }

                        ctx.html()
                            .span()
                            .attr(attr!("class" => "wj-user-info-name"))
                            .inner(log, name);
                    });
            }
        });
}
