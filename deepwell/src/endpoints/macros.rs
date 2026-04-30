/*
 * endpoints/macros.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

macro_rules! parse {
    ($params_method:ident; $params:expr, $error_type:expr $(,)?) => {
        $params.$params_method().or_raise(|| Error::new(
            "failed to read input JSON",
            $error_type,
        ))?
    };

    ($params:expr => $error_type:expr $(,)?) => {
        parse!(parse; $params, $error_type)
    };

    ($params:expr, $error_type:ident $(,)?) => {
        parse!(parse; $params, ErrorType::$error_type)
    };

    ($params:expr $(,)?) => {
        parse!($params, Request)
    };
}

macro_rules! parse_authed {
    ($ctx:expr, $params:expr, $error_type:ident $(,)?) => {{
        let input = parse!($params, $error_type);
        let perms = PermissionService::prefetch_permission_context(
            &$ctx,
            &PrefetchPermissionsInput {
                session_token: input.session_token.clone(),
                site_id: input.site_id,
                page_reference: input.page_reference.clone(),
            },
        )
        .await?;
        $ctx.set_permissions(perms);
        input
    }};

    ($ctx:expr, $params:expr $(,)?) => {
        parse_authed!($ctx, $params, Request)
    };
}

macro_rules! parse_one {
    ($params:expr, $error_type:ident $(,)?) => {
        parse!(one; $params, ErrorType::$error_type)
    };

    ($params:expr => $error_type:expr $(,)?) => {
        parse!(one; $params, $error_type)
    };

    ($params:expr $(,)?) => {
        parse_one!($params, Request)
    };
}
