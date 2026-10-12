//! Runtime expansion for www.wikidot.com system modules.

use super::{
    BTreeMap, CompatHtmlFragments, ConnectionTrait, Error, ErrorType, FromQueryResult,
    LazyLock, LiteralRegionIndex, PageInfo, Reference, Regex, RenderService, Result,
    ResultExt, ServiceContext, Statement, UserService, WikitextSettings,
    escape_list_pages_html_text, now, percent_encode_path_segment,
    wikidot_module_argument, wikidot_module_arguments,
};

const WWW_DELETE_ACCOUNT_INVALID_CODE_HTML: &str = r#"<div class="error-block">Invalid verification code. If you are terminating your account, please start again</div>"#;
const WWW_CREATE_ACCOUNT_ANONYMOUS_HTML: &str = concat!(
    r#"<div class="col-md-5 col-md-offset-7 create-account-col create-account-form"><div class="login-paths"><div class="path with-wikidot"><div class="ca-form"><h1>Create account</h1>"#,
    r#"<form action="/-/register" method="get" name="caform"><input name="fromFrontPage" type="hidden" value="true"><input name="time" type="hidden" value="{}">"#,
    r#"<div class="form-group"><div class="input-group"><span class="input-group-addon"><i class="icon-user"></i></span><input class="text form-control" maxlength="50" name="name" placeholder="username" size="25" type="text" value=""></div></div>"#,
    r#"<div class="form-group"><div class="input-group"><span class="input-group-addon"><i class="icon-envelope"></i></span><input class="text form-control" maxlength="50" name="email" placeholder="email address" size="25" type="text" value=""></div></div>"#,
    r#"<div class="form-group"><div class="input-group"><span class="input-group-addon"><i class="icon-key"></i></span><input class="text form-control" maxlength="64" name="password" placeholder="password" size="15" type="password"></div></div>"#,
    r#"<div class="form-group" style="display:block;position:absolute;left:-9999px; width: 100px; height: 1px;"><div class="input-group"><span class="input-group-addon">Please leave this checkbox blank</span><input name="someData" type="checkbox" value="1"></div></div>"#,
    r#"<button class="button btn btn-danger" type="submit"><i class="icon-signin"></i> Sign up</button><div class="tos-and-pp">By creating an account you accept our <a href="/legal:terms-of-service">Terms of Service</a> and <a href="/legal:privacy-policy">Privacy Policy</a>.</div></form></div></div></div></div>"#,
);
const WWW_NEW_SITE_ANONYMOUS_HTML: &str = concat!(
    r#"<div id="new-site-box"><div style="text-align: center; margin-bottom: 30px;"><i class="icon-ok-sign" style="font-size: 50px; color: #5cb85c; margin-right: 10px;"></i><span style="font-size: 40px; padding-bottom: 10px;"> Get your new Wikidot site</span></div>"#,
    r#"<div class="col-md-8 col-md-offset-2" style="font-size: 24px; font-weight: 200; text-align: center;">Getting your new free Wikidot site is simple and takes about a minute.<br>Please read the <a href="/legal:terms-of-service" target="_blank">Terms of Service</a> before creating a Wiki.<strong>We need you to have an account to create a new site</strong></div>"#,
    r#"<div class="col-lg-6 col-lg-offset-3"><div style="text-align: center; padding-top: 20px;"><div class="form-group"><div class="buttons"><a class="btn btn-primary btn-lg" href="/-/login" style="width: 100%;"><i class="icon-signin"></i> Sign in</a><div class="help-block">if you already have a Wikidot account</div></div></div><div style="text-align: center; font-size: 30px; font-weight: 200; margin-bottom: 20px;">or</div><div class="form-group"><div class="buttons"><a class="btn btn-danger btn-lg" href="/-/register" style="width: 100%;"><i class="icon-signin"></i> Create account</a><div class="help-block">it's free and safe, and only takes a second</div></div></div></div></div></div>"#,
);
const WWW_NEW_SITE_AUTHENTICATED_HTML: &str = concat!(
    r#"<div id="new-site-box"><div style="text-align: center; margin-bottom: 30px;"><i class="icon-ok-sign" style="font-size: 50px; color: #5cb85c; margin-right: 10px;"></i><span style="font-size: 40px; padding-bottom: 10px;"> Get your new Wikidot site</span></div>"#,
    r#"<form id="new-site-form" method="post" action="?/newSite"><div class="form-group"><label>Title</label><input class="form-control" type="text" name="name" required><div class="help-block">Appears on the top-left corner of your Wikidot site.</div></div>"#,
    r#"<div class="form-group"><label>Tagline</label><input class="form-control" type="text" name="subtitle"><div class="help-block">Appears beneath the name.</div></div>"#,
    r#"<div class="form-group"><label>Web address</label><div class="input-group"><input class="form-control" type="text" name="unixname" required><span class="input-group-addon">.wikidot.com</span></div></div>"#,
    r#"<div class="form-group"><label>Language</label><select class="form-control" name="language"><option value="en" selected>English</option></select></div>"#,
    r#"<div class="form-group"><label>Template</label><label><input type="radio" name="template" value="default" checked> Standard wiki</label></div>"#,
    r#"<div class="form-group"><label>Access policy</label><label><input type="radio" name="privacy" value="open" checked> Open</label><label><input type="radio" name="privacy" value="closed"> Closed</label><label><input type="radio" name="privacy" value="private"> Private</label></div>"#,
    r#"<label><input type="checkbox" name="tos" value="1" required> I accept the Terms of Service</label><div class="buttons"><button class="btn btn-primary" type="submit">Create site</button></div></form></div>"#,
);
static CURRENCY_CONVERT_SYSTEM_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)\[\[module CurrencyConvert\]\](?P<body>.*?)\[\[/module\]\]"#)
        .expect("CurrencyConvert system module expression is valid")
});
static WWW_SPECIAL_SYSTEM_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[module\s+(?P<name>CreateAccount|DeleteAccount|FrontSpecialMini|NewSite|SitesTagCloud)\b(?P<head>(?:[^\]"]+|"[^"]*")*)\]\]"#,
    )
    .expect("www special system module expression is valid")
});
#[derive(Debug, FromQueryResult)]
struct WwwFrontSpecialStatsRow {
    pages: i64,
    edits_today: i64,
    people: i64,
    signed_up_today: i64,
}

#[derive(Debug, FromQueryResult)]
struct WwwSiteDirectoryTagRow {
    content: String,
}

impl RenderService {
    pub(super) fn expand_www_currency_convert_system_module(
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
    ) -> String {
        if !settings.enable_page_syntax
            || page_info.site.as_ref() != "www"
            || page_info.page.as_ref() != "plans"
            || !CURRENCY_CONVERT_SYSTEM_MODULE_REGEX.is_match(&wikitext)
        {
            return wikitext;
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in CURRENCY_CONVERT_SYSTEM_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("CurrencyConvert capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let body = captures
                .name("body")
                .expect("CurrencyConvert capture always has a body");
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(body.as_str());
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    fn www_special_module_page(name: &str) -> &'static str {
        if name.eq_ignore_ascii_case("CreateAccount") {
            "start:start"
        } else if name.eq_ignore_ascii_case("DeleteAccount") {
            "action:deleteaccount"
        } else if name.eq_ignore_ascii_case("FrontSpecialMini") {
            "inc:what-is-wikidot"
        } else if name.eq_ignore_ascii_case("NewSite") {
            "new-site"
        } else {
            debug_assert!(name.eq_ignore_ascii_case("SitesTagCloud"));
            "search"
        }
    }

    fn render_www_create_account_anonymous() -> String {
        WWW_CREATE_ACCOUNT_ANONYMOUS_HTML
            .replace("{}", &now().unix_timestamp().to_string())
    }

    pub(super) fn format_www_counter(value: i64) -> String {
        let digits = value.max(0).to_string();
        let mut output = String::with_capacity(digits.len() + digits.len() / 3);
        let first = digits.len() % 3;
        if first > 0 {
            output.push_str(&digits[..first]);
        }
        for start in (first..digits.len()).step_by(3) {
            if !output.is_empty() {
                output.push(' ');
            }
            output.push_str(&digits[start..start + 3]);
        }
        output
    }

    async fn render_www_front_special_mini(ctx: &ServiceContext<'_>) -> Result<String> {
        let make_error =
            || Error::new("failed to render FrontSpecialMini", ErrorType::Render);
        let txn = ctx.transaction();
        let row = WwwFrontSpecialStatsRow::find_by_statement(
            Statement::from_string(
                txn.get_database_backend(),
                "SELECT \
                    (SELECT count(*)::bigint FROM page WHERE deleted_at IS NULL) AS pages, \
                    (SELECT count(*)::bigint FROM page_revision WHERE created_at >= date_trunc('day', now())) AS edits_today, \
                    (SELECT count(*)::bigint FROM known_user) AS people, \
                    (SELECT count(*)::bigint FROM \"user\" WHERE deleted_at IS NULL AND created_at >= date_trunc('day', now())) AS signed_up_today"
                    .to_owned(),
            ),
        )
        .one(txn)
        .await
        .or_raise(make_error)?
        .ok_or_else(|| Error::new("missing FrontSpecialMini aggregate row", ErrorType::Render))?;
        Ok(format!(
            concat!(
                r#"<div class="wikidot-front-special-stats">"#,
                r#"<span style="white-space: nowrap;">pages <span class="number">{}</span></span>   "#,
                r#"<span style="white-space: nowrap;">edits today <span class="number">{}</span></span>   "#,
                r#"<span style="white-space: nowrap;">people <span class="number">{}</span></span>   "#,
                r#"<span style="white-space: nowrap;">signed-up today <span class="number">{}</span></span>"#,
                "</div>"
            ),
            Self::format_www_counter(row.pages),
            Self::format_www_counter(row.edits_today),
            Self::format_www_counter(row.people),
            Self::format_www_counter(row.signed_up_today),
        ))
    }

    async fn render_www_sites_tag_cloud(
        ctx: &ServiceContext<'_>,
        head: &str,
    ) -> Result<Option<String>> {
        let arguments = match wikidot_module_arguments(head) {
            Some(arguments) => arguments,
            None => return Ok(None),
        };
        if arguments.iter().any(|argument| {
            !argument.key.eq_ignore_ascii_case("limit") || argument.op != "="
        }) {
            return Ok(None);
        }
        let limit = wikidot_module_argument(head, "limit")
            .map_or(Some(50usize), |value| value.parse::<usize>().ok())
            .map(|value| value.min(200));
        let Some(limit) = limit else {
            return Ok(None);
        };
        let make_error =
            || Error::new("failed to render SitesTagCloud", ErrorType::Render);
        let txn = ctx.transaction();
        // Imported/platform directory tags have no dedicated storage table in
        // Wikijump. The all-pages metadata key below is the explicit local
        // representation for that site-level read model; absence means an
        // empty local directory, not a replay of Wikidot's captured cloud.
        let rows = WwwSiteDirectoryTagRow::find_by_statement(
            Statement::from_string(
                txn.get_database_backend(),
                "SELECT content FROM page_meta_tag WHERE page_id IS NULL AND name = 'wikidot-site-directory-tags' ORDER BY site_id"
                    .to_owned(),
            ),
        )
        .all(txn)
        .await
        .or_raise(make_error)?;
        let mut counts = BTreeMap::<String, usize>::new();
        for row in rows {
            for tag in row
                .content
                .split(|character: char| {
                    character.is_ascii_whitespace() || character == ','
                })
                .filter(|tag| !tag.is_empty())
            {
                *counts.entry(tag.to_ascii_lowercase()).or_default() += 1;
            }
        }
        let max_count = counts.values().copied().max().unwrap_or(1);
        let mut html = String::from(r#"<div class="sites-tag-cloud-box">"#);
        for (tag, count) in counts.into_iter().take(limit) {
            let weight = if max_count <= 1 {
                0.0
            } else {
                (count.saturating_sub(1)) as f32 / (max_count - 1) as f32
            };
            let font = (25.0 + 75.0 * weight).round() as u8;
            let channel = (128.0 - 64.0 * weight).round() as u8;
            let href_tag = percent_encode_path_segment(&tag);
            html.push_str(&format!(
                r#"<a class="tag" href="/sites-by-tags/tag/{href_tag}" style="font-size: {font}%; color: rgb({channel}, {channel}, {});">{}</a>"#,
                channel.saturating_add(64),
                escape_list_pages_html_text(&tag),
            ));
        }
        html.push_str("</div>");
        Ok(Some(html))
    }

    async fn render_www_special_system_module(
        ctx: &ServiceContext<'_>,
        name: &str,
        head: &str,
        viewer_user_id: Option<i64>,
    ) -> Result<Option<String>> {
        if name.eq_ignore_ascii_case("SitesTagCloud") {
            return Self::render_www_sites_tag_cloud(ctx, head).await;
        }
        if !head.trim().is_empty() {
            return Ok(None);
        }
        if name.eq_ignore_ascii_case("DeleteAccount") {
            return Ok(Some(WWW_DELETE_ACCOUNT_INVALID_CODE_HTML.to_owned()));
        }
        if name.eq_ignore_ascii_case("FrontSpecialMini") {
            return Self::render_www_front_special_mini(ctx).await.map(Some);
        }
        if name.eq_ignore_ascii_case("NewSite") {
            return Ok(Some(
                if viewer_user_id.is_some() {
                    WWW_NEW_SITE_AUTHENTICATED_HTML
                } else {
                    WWW_NEW_SITE_ANONYMOUS_HTML
                }
                .to_owned(),
            ));
        }
        debug_assert!(name.eq_ignore_ascii_case("CreateAccount"));
        let Some(viewer_user_id) = viewer_user_id else {
            return Ok(Some(Self::render_www_create_account_anonymous()));
        };
        let identity = UserService::get(ctx, Reference::Id(viewer_user_id))
            .await?
            .into_public_identity();
        let Some(identity) = identity else {
            return Ok(Some(Self::render_www_create_account_anonymous()));
        };
        Ok(Some(format!(
            concat!(
                r#"<div class="col-md-5 col-md-offset-7 create-account-col create-account-form">"#,
                "<h1>Hello, {}!</h1><p>How are you today?</p>",
                r#"<p>Thank you for using Wikidot. To take a look at your Wikis go to <a href="/-/account">My Account</a></p></div>"#,
            ),
            escape_list_pages_html_text(identity.user_name.as_ref()),
        )))
    }

    pub(super) async fn expand_www_special_system_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        viewer_user_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax
            || page_info.site.as_ref() != "www"
            || !WWW_SPECIAL_SYSTEM_MODULE_REGEX.is_match(&wikitext)
        {
            return Ok(wikitext);
        }
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in WWW_SPECIAL_SYSTEM_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("www special module capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let name = captures
                .name("name")
                .expect("www special module capture always has a name")
                .as_str();
            let full_page_name = match page_info.category.as_deref() {
                Some(category) => format!("{category}:{}", page_info.page),
                None => page_info.page.to_string(),
            };
            if full_page_name != Self::www_special_module_page(name) {
                continue;
            }
            let head = captures.name("head").map_or("", |head| head.as_str());
            let Some(rendered) =
                Self::render_www_special_system_module(ctx, name, head, viewer_user_id)
                    .await?
            else {
                continue;
            };
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(&compat_html.push_block_html(rendered));
            cursor = matched.end();
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }
}

#[cfg(test)]
mod www_special_module_tests {
    use super::{
        RenderService, WWW_DELETE_ACCOUNT_INVALID_CODE_HTML, WWW_NEW_SITE_ANONYMOUS_HTML,
    };
    use crate::utils::now;

    #[test]
    fn anonymous_create_account_form_has_time_hidden_input() {
        let before = now().unix_timestamp();
        let html = RenderService::render_www_create_account_anonymous();
        let after = now().unix_timestamp();
        let time_input = r#"<input name="time" type="hidden" value=""#;
        let value_start =
            html.find(time_input).expect("CreateAccount time input") + time_input.len();
        let value_end = html[value_start..]
            .find("\">")
            .expect("CreateAccount time input value terminator");
        let value = html[value_start..value_start + value_end]
            .parse::<i64>()
            .expect("CreateAccount time input should be Unix seconds");
        assert_eq!(html.matches(time_input).count(), 1);
        assert!(
            value >= before && value <= after,
            "CreateAccount time input should be rendered from the current clock window",
        );
        assert!(!WWW_DELETE_ACCOUNT_INVALID_CODE_HTML.contains(time_input));
        assert!(!WWW_NEW_SITE_ANONYMOUS_HTML.contains(time_input));
    }
}
