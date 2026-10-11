//! Runtime-backed Wikidot module expansion.

mod membership;
mod page_calendar;
mod platform_directory;
mod platform_navigation;
mod rate;
mod rated_pages;
mod tag_cloud;
mod theme_previewer;
pub(crate) use theme_previewer::has_theme_previewer_no_ui;
pub(super) use theme_previewer::{has_theme_previewer_blank, theme_previewer_blank_head};
mod user_lists;
mod www;

use page_calendar::PageCalendarExpansionOptions;
use tag_cloud::TagCloudExpansionOptions;

pub(super) fn wikitext_has_executable_tag_cloud_module(wikitext: &str) -> bool {
    tag_cloud::wikitext_has_executable_tag_cloud_module(wikitext)
}

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::future::Future;
use std::sync::LazyLock;

use regex::Regex;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement, Value};

use super::compat::CompatHtmlFragments;
use super::compat::text_fragments::CompatTextFragments;
use super::diagnostics::{
    CorpusRenderScope, CorpusRenderStage, CorpusRenderTrace, StageGuard,
};
use super::file_modules::expand_file_modules;
use super::list_pages::{CountPagesExpansionOptions, ListPagesRuntimeDisplay};
use super::literal_regions::LiteralRegionIndex;
use super::module_arguments::{
    WikidotModuleArgumentValueKind, wikidot_module_argument, wikidot_module_arguments,
    wikidot_module_arguments_ignoring_bare_flags,
};
use super::native_list_context::{
    collect_unproven_scope_ranges, matching_source_scope_close,
};
use super::new_page_module::{
    NEWPAGE_MODULE_REGEX, NewPageTemplateOption, NewPageTemplateRendering,
    executable_new_page_modules, new_page_template_lookup_slug, render_new_page_module,
};
use super::percent_encoding::percent_encode_path_segment;
use super::rate_actions::RateActionRegistry;
use super::rate_module::{
    render_read_only_rate_module, render_read_only_star_rate_module,
};
use super::runtime_page_queries::find_viewable_list_pages_rows_with_batch_floor;
use super::search_feed::expand_search_feed_modules;
use super::service::{
    MAX_LISTPAGES_RENDER_SCAN_ROWS, PAGECALENDAR_MODULE_REGEX, RATE_MODULE_REGEX,
    RATEDPAGES_MODULE_REGEX, REGISTRY_MODULE_REGEX, RenderService, TAGCLOUD_MODULE_REGEX,
    escape_list_pages_html_attr, escape_list_pages_html_text,
    localized_runtime_module_label, localized_runtime_module_label_from_localizations,
    render_clone_module,
};
use super::site_changes::expand_site_changes_modules;
use super::site_utility_modules::expand_site_utility_modules;
use super::url_arguments::UrlArguments;
use super::user_directory::{MEMBERS_MODULE_REGEX, render_members_module};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::domain::DomainService;
use crate::services::membership::{JoinModuleState, MembershipService};
use crate::services::page_query::{
    AuthorSelector, CategoriesSelector, ComparisonOperation, DateSelector,
    FoundPageFields, IncludedCategories, OrderBySelector, OrderProperty,
    PageParentSelector, PageQuery, PageTypeSelector, PaginationSelector, RangeSelector,
    ScoreSelector, TagCondition,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::relation::{GetSiteMember, RelationObject, RelationReference};
use crate::services::score::ScoreValue;
use crate::services::settings::PageRatingType;
use crate::services::user::User;
use crate::services::view::redirect::escape_wikidot_html_text as escape_redirect_notice_text;
use crate::services::{
    PageDraftService, PageDraftView, PageRevisionService, PageService, RelationService,
    ServiceContext, SiteService, UserService,
};
use crate::types::Reference;
use crate::types::{Action, Permission, RelationType, Resource};
use crate::utils::now;
use ftml::data::PageInfo;
use ftml::settings::WikitextSettings;

const PAGE_CALENDAR_CATEGORY_ERROR: &str = "The requested categories do not (yet) exist.";
const LISTUSERS_UNSUPPORTED_USERS_ERROR: &str =
    r#"Currently only users="." is implemented."#;
const LISTDRAFTS_EMPTY_HTML: &str = r#"<div class="list-drafts-box">
            </div>"#;
const SIMPLETODO_MISSING_ID_ERROR: &str = "The SimpleTodo module must have an id.";
const REDIRECT_MISSING_DESTINATION_HTML: &str = r#"<div class="error-block">No redirection destination specified. Please use the destination="page-name" or destination="url" attribute.</div>"#;
const SENDINVITATIONS_DISABLED_ERROR_HTML: &str = r#"<div class="error-block">Inviting users has been disabled due to severe abuse. Admins can still send email invitations via <a href="/_admin">site admin dashboard</a>.</div>"#;
const ANONYMOUS_NOTIFICATIONS_UNSUBSCRIBE_INVALID_TOKEN_HTML: &str =
    r#"<div class="error-block">Invalid indentification token.</div>"#;
const DASHBOARD_NOT_ALLOWED_HTML: &str =
    r#"<div class="error-block">Not allowed. Error.</div>"#;
const USERINFO_NO_USER_HTML: &str =
    r#"<div class="error-block">No user specified.</div>"#;
const SEARCHUSERS_DISABLED_HTML: &str = r#"<div class="error-block">User search has been (temporarily) disabled. Sorry!</div>"#;
const THEME_PREVIEWER_PREVIEW_ERROR_HTML: &str = r#"<div class="error-block">Preview mode error: please contact Wikidot.com for a better error message</div>"#;
const MEMBERSHIP_EMAIL_INVITATION_MISSING_HTML: &str = concat!(
    r#"<div id="membership-email-invitation-box">"#,
    "\n\t\n\t\t\t<p>\n\t\t\t",
    "Sorry, the invitation could not be found. It might have been canceled by the sender, aleady",
    "\n\t\t\tused by someone (you?) or the URL link that you were supposed to copy",
    "\n\t\t\tfrom the invitation email might be corrupted somehow.",
    "\n\t\t</p>\t\n\t</div>",
);

fn render_membership_email_invitation_print_user(
    user_id: i64,
    name: &str,
    profile_url: &str,
    image: bool,
) -> String {
    let name_text = escape_list_pages_html_text(name);
    if profile_url.is_empty() {
        return format!(r#"<span class="printuser">{name_text}</span>"#);
    }
    let profile = escape_list_pages_html_attr(profile_url);
    if !image {
        return format!(
            concat!(
                r#"<span class="printuser"><a href="{profile}" "#,
                r#"onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">"#,
                "{name_text}</a></span>",
            ),
            profile = profile,
            user_id = user_id,
            name_text = name_text,
        );
    }
    let name_attr = escape_list_pages_html_attr(name);
    format!(
        concat!(
            r#"<span class="printuser avatarhover"><a href="{profile}" "#,
            r#"onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">"#,
            r#"<img class="small" src="https://www.wikidot.com/avatar.php?userid={user_id}&amp;size=small" "#,
            r#"alt="{name_attr}" style="background-image:url(https://www.wikidot.com/userkarma.php?u={user_id})"/></a>"#,
            r#"<a href="{profile}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">"#,
            "{name_text}</a></span>",
        ),
        profile = profile,
        user_id = user_id,
        name_attr = name_attr,
        name_text = name_text,
    )
}

fn render_membership_email_invitation_valid(
    invitation: &crate::services::membership::MembershipEmailInvitationView,
    viewer: Option<(i64, String, String)>,
    hash: &str,
) -> String {
    let authenticated = viewer.is_some();
    let greeting = match viewer {
        Some((user_id, name, profile_url)) => {
            render_membership_email_invitation_print_user(
                user_id,
                &name,
                &profile_url,
                false,
            )
        }
        None => escape_list_pages_html_text(&invitation.recipient_name),
    };
    let sender = render_membership_email_invitation_print_user(
        invitation.sender_user_id,
        &invitation.sender_user_name,
        &invitation.sender_profile_url,
        true,
    );
    let site_name = escape_list_pages_html_text(&invitation.site_name);
    let site_domain = format!("{}.wikidot.com", invitation.site_slug);
    let site_domain_attr = escape_list_pages_html_attr(&site_domain);
    let hash = escape_list_pages_html_attr(hash);
    let actor_control = if authenticated {
        format!(
            concat!(
                r#"<p style="padding: 1em; font-size: 180%; text-align: center;font-weight: bold;line-spacing: 120%;">"#,
                r#"<a href="javascript:;" onclick="WIKIDOT.modules.MembershipEmailInvitationModule.listeners.accept(event, '{hash}')">accept invitation</a>"#,
                "</p>",
            ),
            hash = hash,
        )
    } else {
        concat!(
            "<p>Please create an account (or log in) before you can accept the invitation.</p>",
            r#"<table style="margin: 1em auto"><tr>"#,
            r#"<td style="text-align: center; padding: 1em"><div style="font-size: 180%; font-weight: bold;">"#,
            r#"<a href="javascript:;" onclick="WIKIREQUEST.createAccountSkipCongrats=true;WIKIDOT.page.listeners.loginClick(event)">log in</a>"#,
            "</div><p>if you already have an account at Wikidot</p></td>",
            r#"<td style="padding: 1em; font-size: 140%">or</td>"#,
            r#"<td style="text-align: center; padding: 1em"><div style="font-size: 180%; font-weight: bold;">"#,
            r#"<a href="javascript:;" onclick="WIKIREQUEST.createAccountSkipCongrats=true; WIKIDOT.page.listeners.createAccount(event)">create a new account</a>"#,
            "</div></td></tr></table>",
        )
        .to_owned()
    };
    format!(
        concat!(
            r#"<div id="membership-email-invitation-box">"#,
            "<h2><span>Hi, {greeting}!</span></h2>",
            "<p>It seems you got an invitation from our user {sender} to become a member of his/her Wiki Website ",
            r#"<b>{site_name}</b> at <a href="http://{site_domain_attr}" target="_blank">http://{site_domain}</a>.</p>"#,
            "<p>All you have to do is to accept the invitation and we will instantly add you to members of this Site.</p>",
            "{actor_control}</div>",
        ),
        greeting = greeting,
        sender = sender,
        site_name = site_name,
        site_domain_attr = site_domain_attr,
        site_domain = escape_list_pages_html_text(&site_domain),
        actor_control = actor_control,
    )
}
const WHOINVITED_FORM_HTML: &str = concat!(
    r#"<form action="dummy" id="who-invited-form" onsubmit="WIKIDOT.modules.WhoInvitedModule.listeners.lookUp(event)">"#,
    "\n\t",
    r#"<table class="form">"#,
    "\n\t\t<tr>\n\t\t\t<td>\n\t\t\t\tWho invited this guy?\t\t\t</td>\n\t\t\t<td>\n\t\t\t\t",
    r#"<div class="autocomplete-container" style="width: 20em; padding-top: 3px;">"#,
    "\n\t\t\t\t\t\t",
    r#"<input type="text" id="user-lookup" size="30" class="autocomplete-input text"/>"#,
    "\n\t\t\t\t\t\t",
    r#"<div id="user-lookup-list" class="autocomplete-list"></div>"#,
    "\n\t\t\t\t</div>\n\t\t\t\t",
    r#"<div class="sub">"#,
    "\n\t\t\t\t\tType name of the user\t\t\t\t</div>\n\t\t\t</td>\n\t\t</tr>\n\t</table>\n\t",
    r#"<div class="buttons">"#,
    "\n\t\t",
    r#"<input type="submit" value="look up"/>"#,
    "\n\t</div>\n</form>\n\n",
    r#"<div id="who-invited-results-box">"#,
    "\n\n</div>",
);
const MEMBERSHIP_BY_PASSWORD_ANONYMOUS_HTML: &str = concat!(
    r#"<div id="membership-by-password-box">"#,
    "\n\t\t\t<p>\n\t\t\tPlease create an account and/or sign in first.\t\t</p>",
    "\n\t\t",
    r#"<table style="margin: 1em auto">"#,
    "\n\t\t\t<tr>\n\t\t\t\t",
    r#"<td style="text-align: center; padding: 1em">"#,
    "\n\t\t\t\t\t",
    r#"<div style="font-size: 180%; font-weight: bold;">"#,
    "\n\t\t\t\t\t\t",
    r#"<a href="javascript:;" onclick="WIKIDOT.page.listeners.loginClick(event)""#,
    "\n\t\t\t\t\t\t\t>Sign in</a>\n\t\t\t\t\t</div>\n\t\t\t\t\t<p>\t\n\t\t\t\t\t\tif you already have a Wikidot.com account\t\t\t\t\t</p>\n\t\t\t\t</td>",
    "\n\t\t\t\t",
    r#"<td style="padding: 1em; font-size: 140%">"#,
    "\n\t\t\t\t\tor\t\t\t\t</td>\n\t\t\t\t",
    r#"<td style="text-align: center; padding: 1em">"#,
    "\n\t\t\t\t\t",
    r#"<div style="font-size: 180%; font-weight: bold;">"#,
    "\n\t\t\t\t\t\t",
    r#"<a href="javascript:;"  onclick="WIKIREQUEST.createAccountSkipCongrats=true; WIKIDOT.page.listeners.createAccount(event)""#,
    "\n\t\t\t\t\t\t\t>Create a new account</a>\n\t\t\t\t\t</div>\n\t\t\t\t\t<p>\n\t\t\t\t\t\tit is worth it and is free\t\t\t\t\t</p>\n\t\t\t\t</td>\n\t\t\t</tr>\n\t\t</table>\n\t\n</div>",
);
const MEMBERSHIP_BY_PASSWORD_MEMBER_HTML: &str = concat!(
    r#"<div id="membership-by-password-box">"#,
    "\n\t\t\t",
    r#"<div class="error-block">"#,
    "\n\t\t\tYou can not apply.<br/>\n\t\t\t\t\t\t\t\t\t\tIt seems you already are a member of this site.\t\t\t\t\t\t\t\t</div>\n\t\n</div>",
);
const MEMBERSHIP_BY_PASSWORD_FORM_HTML: &str = concat!(
    "<div id=\"membership-by-password-box\">\n",
    "<div id=\"mbp-error\" class=\"error-block\" style=\"display: none;\">\n",
    "\t<div>\n\t\tThe password is not valid.\t</div>\n</div>\n\n",
    "<form id=\"membership-by-password-form\" onsubmit=\"return false;\" action=\"dummy.html\" method=\"get\">\n",
    "\t<table class=\"form\">\n\t\t<tr>\n\t\t\t<td>\n\t\t\t\tPassword:\n\t\t\t</td>\n",
    "\t\t\t<td>\n\t\t\t\t<input class=\"text\" type=\"password\" name=\"password\" size=\"40\" maxlength=\"50\"/><br/>\n\t\t\t</td>\n\t\t</tr>\n\t</table>\n",
    "\t<div class=\"buttons\">\n\t\t<input id=\"mbp-apply\" type=\"button\" value=\"Apply\" onclick=\"WIKIDOT.modules.MembershipByPasswordModule.listeners.apply(event)\"/>\n\t</div>\n",
    "</form>\n\n</div>",
);
const MEMBERSHIP_BY_PASSWORD_DISABLED_HTML: &str = concat!(
    r#"<div id="membership-by-password-box">"#,
    "\n\t\t\t",
    r#"<div class="error-block">"#,
    "\n\t\t\tYou can not apply.<br/>\n\t\t\tMembership via password is not enabled for this site.\n\t\t</div>\n\t\n</div>",
);
const MEMBERSHIP_APPLY_ANONYMOUS_HTML: &str = concat!(
    r#"<div id="membership-apply-box">"#,
    "\n\t<p>You need to have a Wikidot.com account and be signed to apply for membership.</p>",
    r#"<table style="margin: 1em auto"><tr>"#,
    r#"<td style="text-align: center; padding: 1em"><div style="font-size: 180%; font-weight: bold;">"#,
    r#"<a href="javascript:;" onclick="WIKIDOT.page.listeners.loginClick(event)">Sign in</a>"#,
    "</div><p>if you already have a Wikidot.com account</p></td>",
    r#"<td style="padding: 1em; font-size: 140%">or</td>"#,
    r#"<td style="text-align: center; padding: 1em"><div style="font-size: 180%; font-weight: bold;">"#,
    r#"<a href="javascript:;" onclick="WIKIREQUEST.createAccountSkipCongrats=true; WIKIDOT.page.listeners.createAccount(event)">Create a new account</a>"#,
    "</div><p>it is worth it and is free</p></td></tr></table>\n</div>",
);
const MEMBERSHIP_APPLY_FORM_HTML: &str = concat!(
    "<div id=\"membership-apply-box\">\n\n",
    "<form id=\"membership-by-apply-form\">\n",
    "\t<table class=\"form\">\n\t\t<tr>\n\t\t\t<td>\n\t\t\t\tApplication text:\n\t\t\t</td>\n",
    "\t\t\t<td>\n\t\t\t\t<textarea name=\"comment\" rows=\"5\" cols=\"50\" id=\"membership-by-apply-text\"></textarea>\n",
    "\t\t\t\t<div class=\"sub\" style=\"text-align: center;\">\n\t\t\t\t\t(<span id=\"membership-by-apply-text-left\"></span> characters left)\n\t\t\t\t</div>\n\t\t\t</td>\n\t\t</tr>\n\t</table>\n\n",
    "\t<div class=\"buttons\">\n\t\t<input id=\"mba-apply\" type=\"button\" value=\"Apply\"/>\n\t</div>\n",
    "</form>\n</div>",
);
const MEMBERSHIP_APPLY_DISABLED_HTML: &str = concat!(
    "<div id=\"membership-apply-box\"><div class=\"error-block\">",
    "You can not apply.<br/>Membership via application is not enabled for this site.",
    "</div></div>",
);
const MEMBERSHIP_APPLY_MEMBER_HTML: &str = concat!(
    "<div id=\"membership-apply-box\"><div class=\"error-block\">",
    "You can not apply.<br/>It seems you already are a member of this site.",
    "</div></div>",
);
const MEMBERSHIP_APPLY_ALREADY_APPLIED_HTML: &str = concat!(
    "<div id=\"membership-apply-box\"><div class=\"error-block\">",
    "You can not apply.<br/>It seems you have already applied for membership.",
    "</div></div>",
);

static LISTUSERS_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[module\s+ListUsers(?P<head>(?:[^\]"]+|"[^"]*")*)\]\](?P<body>.*?)\[\[/module\]\]"#)
        .expect("ListUsers module expression is valid")
});
static LISTDRAFTS_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[module\s+ListDrafts(?P<head>(?:[^\]"]+|"[^"]*")*)\]\]"#)
        .expect("ListDrafts module expression is valid")
});
static SIMPLETODO_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[module\s+SimpleToDo\b(?P<head>(?:[^\]"]+|"[^"]*")*)\]\]"#)
        .expect("SimpleToDo module expression is valid")
});
static SENDINVITATIONS_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[module\s+SendInvitations\b(?:[^\]"]+|"[^"]*")*\]\]"#)
        .expect("SendInvitations module expression is valid")
});
static STATIC_ACCOUNT_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[module\s+(?P<name>AnonymousNotificationsUnsubscribe|Dashboard|UserInfo|SearchUsers|Watchers|WhoInvited|ThemePreviewer|MembershipEmailInvitation)\b(?P<head>(?:[^\]"]+|"[^"]*")*)\]\]"#,
    )
    .expect("static account module expression is valid")
});
static MEMBERSHIPBYPASSWORD_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[module\s+MembershipByPassword\b(?P<head>(?:[^\]"]+|"[^"]*")*)\]\]"#,
    )
    .expect("MembershipByPassword module expression is valid")
});
static MEMBERSHIPAPPLY_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[module\s+MembershipApply\b(?P<head>(?:[^\]"]+|"[^"]*")*)\]\]"#)
        .expect("MembershipApply module expression is valid")
});
static AD_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[module\s+Ad\b(?:[^\]"]+|"[^"]*")*\]\]"#)
        .expect("Ad module expression is valid")
});
static ADSENSEUNIT_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[module\s+AdSenseUnit\b(?:[^\]"]+|"[^"]*")*\]\]"#)
        .expect("AdSenseUnit module expression is valid")
});
static RUNTIME_MODULE_RESIDUAL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[module[ \t]+(?P<name>Redirect|NewPage|PagesByTag|LoginStatus|NaviBar|FooterBar|PageOptionsBottom|AdModuleAboveContent|AdModuleBelowContent|AdModuleAboveSidebar|AdModuleBelowSidebar|AdModuleBelowFooter)\b(?P<head>(?:[^\]"'\r\n]+|"[^"]*"|'[^']*')*)\]\]"#,
    )
    .expect("runtime module residual expression is valid")
});
static REDIRECT_SINGLE_QUOTED_DESTINATION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)^[ \t]*destination[ \t]*=[ \t]*'[^']*'[ \t]*$"#)
        .expect("single-quoted Redirect destination expression is valid")
});

#[derive(Default)]
struct MembershipByPasswordResultCache {
    rendered: Option<Option<&'static str>>,
}

impl MembershipByPasswordResultCache {
    async fn get_or_init<F, Fut>(&mut self, load: F) -> Result<Option<&'static str>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Option<&'static str>>>,
    {
        if let Some(rendered) = self.rendered {
            return Ok(rendered);
        }

        let rendered = load().await?;
        self.rendered = Some(rendered);
        Ok(rendered)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SecondaryRuntimeModuleExpansionOptions<'a> {
    pub(super) current_site_id: Option<i64>,
    pub(super) current_page_id: Option<i64>,
    pub(super) viewer_user_id: Option<i64>,
    pub(super) page_preview: bool,
    pub(super) url: UrlArguments<'a>,
    pub(super) trace: Option<(&'a CorpusRenderTrace, CorpusRenderScope)>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RateModuleContext {
    pub(super) rating_type: PageRatingType,
    pub(super) score: ftml::data::ScoreValue,
    pub(super) rating_votes: Option<i64>,
}

#[derive(Clone, Debug)]
struct ListUsersViewer {
    number: String,
    title: String,
    name: String,
}

fn substitute_list_users_variables(body: &str, viewer: &ListUsersViewer) -> String {
    body.replace("%%number%%", &viewer.number)
        .replace("%%title%%", &viewer.title)
        .replace("%%name%%", &viewer.name)
}

async fn resolve_list_users_viewer(
    ctx: &ServiceContext<'_>,
    viewer_user_id: Option<i64>,
) -> Result<Option<ListUsersViewer>> {
    let Some(user_id) = viewer_user_id else {
        return Ok(None);
    };
    let Some(user) = UserService::get_optional(ctx, Reference::Id(user_id)).await? else {
        return Ok(None);
    };

    Ok(Some(match user {
        User::Wikijump(user) => ListUsersViewer {
            number: user.user_id.to_string(),
            title: user.name,
            name: user.slug,
        },
        User::Wikidot(user) => {
            let number = user.user_id.to_string();
            let title = user
                .name
                .clone()
                .or_else(|| user.slug.clone())
                .unwrap_or_else(|| number.clone());
            let name = user.slug.or(user.name).unwrap_or_else(|| number.clone());
            ListUsersViewer {
                number,
                title,
                name,
            }
        }
    }))
}

fn render_join_module(head: &str) -> String {
    let button = wikidot_join_argument(head, "button")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Join");
    let class = wikidot_join_argument(head, "class")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("join-box");
    format!(
        concat!(
            r#"<div class="{class}">"#,
            r#"<a href="javascript:;" onclick="WIKIDOT.page.listeners.join(event, 'unified')">{button}</a>"#,
            "</div>",
        ),
        class = escape_list_pages_html_attr(class),
        button = escape_list_pages_html_text(button),
    )
}

fn wikidot_join_argument<'a>(head: &'a str, name: &str) -> Option<&'a str> {
    wikidot_module_arguments(head)?
        .into_iter()
        .rev()
        .find(|argument| {
            argument.key.eq_ignore_ascii_case(name)
                && argument.op == "="
                && argument.value_kind == WikidotModuleArgumentValueKind::DoubleQuoted
        })
        .map(|argument| argument.value)
}

pub(crate) fn join_module_action_count(wikitext: &str) -> usize {
    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(wikitext);
    REGISTRY_MODULE_REGEX
        .captures_iter(wikitext)
        .filter(|captures| {
            let matched = captures
                .get(0)
                .expect("a registry module capture always has a complete match");
            !literal_regions.contains(matched.start())
                && captures
                    .name("name")
                    .is_some_and(|name| name.as_str().eq_ignore_ascii_case("Join"))
        })
        .count()
}

pub(crate) fn membership_apply_action_count(wikitext: &str) -> usize {
    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(wikitext);
    MEMBERSHIPAPPLY_MODULE_REGEX
        .captures_iter(wikitext)
        .filter(|captures| {
            let matched = captures
                .get(0)
                .expect("a MembershipApply capture always has a complete match");
            !literal_regions.contains(matched.start())
                && captures
                    .name("head")
                    .is_none_or(|head| head.as_str().trim().is_empty())
        })
        .count()
}

pub(crate) fn membership_by_password_action_count(wikitext: &str) -> usize {
    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(wikitext);
    MEMBERSHIPBYPASSWORD_MODULE_REGEX
        .captures_iter(wikitext)
        .filter(|captures| {
            let matched = captures
                .get(0)
                .expect("a MembershipByPassword capture always has a complete match");
            !literal_regions.contains(matched.start())
                && captures
                    .name("head")
                    .is_none_or(|head| head.as_str().trim().is_empty())
        })
        .count()
}

pub(crate) fn membership_email_invitation_action_count(wikitext: &str) -> usize {
    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(wikitext);
    STATIC_ACCOUNT_MODULE_REGEX
        .captures_iter(wikitext)
        .filter(|captures| {
            let matched = captures
                .get(0)
                .expect("a static account module capture always has a complete match");
            !literal_regions.contains(matched.start())
                && captures.name("name").is_some_and(|name| {
                    name.as_str()
                        .eq_ignore_ascii_case("MembershipEmailInvitation")
                })
                && captures
                    .name("head")
                    .is_none_or(|head| head.as_str().trim().is_empty())
        })
        .count()
}

fn render_simpletodo_module(head: &str, index: usize) -> String {
    let Some(list_id) =
        wikidot_module_argument(head, "id").filter(|value| !value.trim().is_empty())
    else {
        return format!(
            r#"<div class="error-block">{SIMPLETODO_MISSING_ID_ERROR}</div>"#
        );
    };
    let label = escape_list_pages_html_text(list_id);

    format!(
        concat!(
            r#"<div class="simpletodo-box" id="simpletodo_{index}">"#,
            r#"<div class="title">Here is a place for your title</div>"#,
            r#"<table class="simpletodo-format-table"><tr><td>"#,
            r#"<div class="simpletodo-sub-box" id="simpletodo_sub_{index}">"#,
            r#"<div class="task"><span class="checkbox"><input type="checkbox" class="checkbox"/></span>"#,
            r#"<span><span class="text">Click me to edit !</span></span>"#,
            r#"<span class="follow-link"><a class="icon1" aria-disabled="true"><span>Follow link</span></a></span>"#,
            r#"<span class="options"></span></div>"#,
            r#"<div class="task"><span class="checkbox"><input type="checkbox" class="checkbox"/></span>"#,
            r#"<span><span class="text">Drag me !</span></span>"#,
            r#"<span class="follow-link"><a class="icon1" aria-disabled="true">Follow Link</a></span>"#,
            r#"<span class="options"></span></div>"#,
            r#"</div></td></tr></table>"#,
            r#"<div class="bottom-options"></div>"#,
            r#"<div class="label">{label}</div></div>"#,
            "\n",
            r#"<div id="simpletodo-data">"#,
            "\n",
            r#"<span id="simpletodo-data-title">Here is a place for your title</span>"#,
            "\n",
            r#"<span id="simpletodo-data-itemtext">Click me to edit !</span>"#,
            "\n",
            r#"<span id="simpletodo-data-edit-permission">false</span>"#,
            "\n",
            r#"</div>"#,
        ),
        index = index,
        label = label,
    )
}

async fn resolve_new_page_templates(
    ctx: &ServiceContext<'_>,
    current_site_id: Option<i64>,
    names: &[&str],
) -> Result<NewPageTemplateRendering> {
    if names.is_empty() {
        return Ok(NewPageTemplateRendering::None);
    }

    let Some(site_id) = current_site_id else {
        return Ok(NewPageTemplateRendering::Error(format!(
            "Template \"{}\" can not be found.",
            names[0],
        )));
    };

    let mut options = Vec::with_capacity(names.len());
    for name in names {
        let Some(lookup_slug) = new_page_template_lookup_slug(name) else {
            return Ok(NewPageTemplateRendering::Error(format!(
                "\"{name}\" is not in the \"template:\" category.",
            )));
        };
        let Some(page) = PageService::get_optional(
            ctx,
            site_id,
            Reference::Slug(Cow::Owned(lookup_slug)),
        )
        .await?
        else {
            return Ok(NewPageTemplateRendering::Error(format!(
                "Template \"{name}\" can not be found.",
            )));
        };
        let can_view = PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id: None,
                site_id,
                page_reference: Some(Reference::Id(page.page_id)),
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(page.page_category_id)),
                action: Action::View,
            },
        )
        .await?;
        if !can_view {
            return Ok(NewPageTemplateRendering::Error(format!(
                "Template \"{name}\" can not be found.",
            )));
        }
        let revision =
            PageRevisionService::get_latest(ctx, site_id, page.page_id).await?;
        options.push(NewPageTemplateOption {
            page_id: page.page_id,
            title: revision.title,
        });
    }

    Ok(match options.len() {
        0 => NewPageTemplateRendering::None,
        1 => NewPageTemplateRendering::Single(
            options.into_iter().next().expect("len was checked above"),
        ),
        _ => NewPageTemplateRendering::Multiple(options),
    })
}

fn is_literal_runtime_module_residual(name: &str) -> bool {
    ["Redirect", "NewPage", "PagesByTag"]
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

/// Preview-only redirect notice for an evidenced double-quoted destination.
///
/// Live PagePreview renders the redirect notice for exact
/// `destination="..."` heads with a nonempty value; an empty destination
/// renders the missing-destination error instead. Single quotes stay literal
/// in preview and use the missing-destination error in saved renders.
fn redirect_preview_destination_notice(head: &str) -> Option<String> {
    let destination = head
        .trim()
        .strip_prefix("destination=\"")?
        .strip_suffix('"')?;
    (!destination.is_empty()).then(|| {
        format!(
            concat!(
                "<div class=\"error-block\">\n",
                "\tThis is the Redirect module that redirects the browser ",
                "directly to the &quot;{}&quot; page.\n",
                "</div>",
            ),
            escape_redirect_notice_text(destination),
        )
    })
}

fn render_unavailable_page_module(name: &str) -> String {
    format!(
        concat!(
            r#"<div class="error-block">[[module <em>{}</em>]] No such module, please "#,
            r#"<a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a>"#,
            " and fix this page.</div>",
        ),
        escape_list_pages_html_text(name),
    )
}

impl RenderService {
    pub(super) fn finalize_runtime_module_residuals(
        wikitext: String,
        settings: &WikitextSettings,
        page_preview: bool,
        compat_text: &mut CompatTextFragments,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        if !settings.enable_page_syntax
            || !RUNTIME_MODULE_RESIDUAL_REGEX.is_match(&wikitext)
        {
            return wikitext;
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in RUNTIME_MODULE_RESIDUAL_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a residual module capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let name = captures
                .name("name")
                .expect("a residual module capture always has a name")
                .as_str();
            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            let replacement = if name.eq_ignore_ascii_case("Redirect")
                && (head.trim().is_empty()
                    || head.trim() == "destination=\"\""
                    || (!page_preview
                        && REDIRECT_SINGLE_QUOTED_DESTINATION_REGEX.is_match(head)))
            {
                compat_html.push_block_html(REDIRECT_MISSING_DESTINATION_HTML.to_owned())
            } else if is_literal_runtime_module_residual(name) {
                if page_preview {
                    match redirect_preview_destination_notice(head) {
                        Some(notice) => compat_html.push_block_html(notice),
                        None => compat_text.push_escaped_html_text(matched.as_str()),
                    }
                } else {
                    compat_text.push_escaped_html_text(matched.as_str())
                }
            } else if head.trim().is_empty() {
                compat_html.push_block_html(render_unavailable_page_module(name))
            } else {
                continue;
            };

            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(&replacement);
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    pub(super) fn expand_registry_modules_with_registry(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        // NewPage is expanded by the runtime-backed pass above. Keeping it
        // out of this context-free fallback is what lets that pass preserve
        // over-budget or otherwise unsupported modules literally. Members
        // has its own site-scoped directory pass and is not in this registry.
        Self::expand_registry_modules_matching(wikitext, settings, compat_html, |name| {
            !name.eq_ignore_ascii_case("NewPage") && !name.eq_ignore_ascii_case("Join")
        })
    }

    fn expand_registry_modules_matching(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
        mut should_expand: impl FnMut(&str) -> bool,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        // Keep one index over the authored source for the complete pass. A replacement must not expose a later candidate that the original literal, comment, or tag boundaries protected, so malformed cross-boundary input remains fail closed.
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in REGISTRY_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a module capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let name = captures
                .name("name")
                .expect("a registry module capture always has a name")
                .as_str();
            if !should_expand(name) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            let rendered = if name.eq_ignore_ascii_case("NewPage") {
                render_new_page_module(
                    head,
                    NewPageTemplateRendering::None,
                    "Name of the new page",
                )
            } else if name.eq_ignore_ascii_case("Clone") {
                render_clone_module(head)
            } else {
                debug_assert!(name.eq_ignore_ascii_case("Join"));
                render_join_module(head)
            };
            let marker = if name.eq_ignore_ascii_case("Join") {
                compat_html.push_block_html(rendered)
            } else {
                compat_html.push_html(rendered)
            };
            output.push_str(&marker);
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    #[cfg(test)]
    pub(super) fn expand_new_page_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("NewPage"),
        );
        fragments.restore(&protected)
    }

    pub(super) async fn expand_new_page_modules_with_registry(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        language: &str,
        current_site_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !NEWPAGE_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let accessible_name = localized_runtime_module_label(
            ctx,
            language,
            "wiki-page-module-new-page-name",
            "Name of the new page",
        );
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for module in executable_new_page_modules(&wikitext) {
            output.push_str(&wikitext[cursor..module.source_range.start]);
            let Some(template_names) = module.template_names.as_deref() else {
                output.push_str(&wikitext[module.source_range.clone()]);
                cursor = module.source_range.end;
                continue;
            };
            let templates =
                resolve_new_page_templates(ctx, current_site_id, template_names).await?;
            let rendered =
                render_new_page_module(module.head, templates, &accessible_name);
            output.push_str(&compat_html.push_html(rendered));
            cursor = module.source_range.end;
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }

    #[cfg(test)]
    pub(super) fn expand_clone_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("Clone"),
        );
        fragments.restore(&protected)
    }

    #[cfg(test)]
    pub(super) fn expand_join_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("Join"),
        );
        fragments.restore(&protected)
    }

    async fn expand_join_modules_for_view(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        viewer_user_id: Option<i64>,
        preview: bool,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax {
            return Ok(wikitext);
        }
        let Some(site_id) = current_site_id else {
            return Ok(wikitext);
        };
        let actor_state =
            MembershipService::actor_state(ctx, site_id, viewer_user_id).await?;
        let show = if preview {
            MembershipService::join_module_preview_state(actor_state)
        } else {
            MembershipService::join_module_state(actor_state)
        } == JoinModuleState::Show;
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in REGISTRY_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a registry module capture always has a complete match");
            let is_join = captures
                .name("name")
                .is_some_and(|name| name.as_str().eq_ignore_ascii_case("Join"));
            if !is_join || literal_regions.contains(matched.start()) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            if show {
                let head = captures.name("head").map_or("", |head| head.as_str());
                output.push_str(&compat_html.push_block_html(render_join_module(head)));
            }
            cursor = matched.end();
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }

    async fn expand_members_modules_with_directory(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !MEMBERS_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }
        let Some(site_id) = current_site_id else {
            return Ok(wikitext);
        };

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        let mut module_index = 0;
        for captures in MEMBERS_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a Members capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            module_index += 1;
            let head = captures.name("head").map_or("", |head| head.as_str());
            let Some(rendered) =
                render_members_module(ctx, site_id, head, module_index).await?
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

    fn expand_simpletodo_modules(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        if !settings.enable_page_syntax || !SIMPLETODO_MODULE_REGEX.is_match(&wikitext) {
            return wikitext;
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        let mut simpletodo_index = 0usize;
        for captures in SIMPLETODO_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a SimpleToDo capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            let head = captures.name("head").map_or("", |head| head.as_str());
            let rendered = render_simpletodo_module(head, simpletodo_index);
            if wikidot_module_argument(head, "id")
                .is_some_and(|value| !value.trim().is_empty())
            {
                simpletodo_index += 1;
            }
            output.push_str(&compat_html.push_block_html(rendered));
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    fn expand_send_invitations_modules(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        if !settings.enable_page_syntax
            || !SENDINVITATIONS_MODULE_REGEX.is_match(&wikitext)
        {
            return wikitext;
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for matched in SENDINVITATIONS_MODULE_REGEX.find_iter(&wikitext) {
            if literal_regions.contains(matched.start()) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(
                &compat_html
                    .push_block_html(SENDINVITATIONS_DISABLED_ERROR_HTML.to_owned()),
            );
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    fn render_static_account_module(name: &str) -> &'static str {
        if name.eq_ignore_ascii_case("AnonymousNotificationsUnsubscribe") {
            ANONYMOUS_NOTIFICATIONS_UNSUBSCRIBE_INVALID_TOKEN_HTML
        } else if name.eq_ignore_ascii_case("Dashboard") {
            DASHBOARD_NOT_ALLOWED_HTML
        } else if name.eq_ignore_ascii_case("UserInfo") {
            USERINFO_NO_USER_HTML
        } else if name.eq_ignore_ascii_case("SearchUsers") {
            SEARCHUSERS_DISABLED_HTML
        } else if name.eq_ignore_ascii_case("Watchers") {
            ""
        } else if name.eq_ignore_ascii_case("ThemePreviewer") {
            THEME_PREVIEWER_PREVIEW_ERROR_HTML
        } else if name.eq_ignore_ascii_case("MembershipEmailInvitation") {
            MEMBERSHIP_EMAIL_INVITATION_MISSING_HTML
        } else {
            debug_assert!(name.eq_ignore_ascii_case("WhoInvited"));
            WHOINVITED_FORM_HTML
        }
    }

    fn expand_static_account_modules(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        if !settings.enable_page_syntax
            || !STATIC_ACCOUNT_MODULE_REGEX.is_match(&wikitext)
        {
            return wikitext;
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in STATIC_ACCOUNT_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a static account module capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let name = captures
                .name("name")
                .expect("a static account module capture always has a name")
                .as_str();
            let head = captures.name("head").map_or("", |head| head.as_str());
            if name.eq_ignore_ascii_case("ThemePreviewer")
                && theme_previewer_blank_head(head)
            {
                output.push_str(&wikitext[cursor..matched.start()]);
                cursor = matched.end();
                continue;
            }
            let opaque_token_surface = name
                .eq_ignore_ascii_case("AnonymousNotificationsUnsubscribe")
                || name.eq_ignore_ascii_case("MembershipEmailInvitation");
            if !head.trim().is_empty() && !opaque_token_surface {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            let rendered = Self::render_static_account_module(name);
            if rendered.is_empty() && name.eq_ignore_ascii_case("Watchers") {
                // Wikidot leaves an empty paragraph for an executed
                // Wikidot's empty Watchers block leaves five newline bytes
                // between the surrounding paragraphs in PagePreview. Keep
                // that observed block boundary even though the module has no
                // visible body.
                output.push_str(
                    &compat_html.push_block_html("<p>\n\n\n\n\n</p>".to_owned()),
                );
            } else if !rendered.is_empty() {
                output.push_str(&compat_html.push_block_html(rendered.to_owned()));
            }
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    fn expand_ad_modules(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        if !settings.enable_page_syntax
            || (!AD_MODULE_REGEX.is_match(&wikitext)
                && !ADSENSEUNIT_MODULE_REGEX.is_match(&wikitext))
        {
            return wikitext;
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        let mut matches = AD_MODULE_REGEX
            .find_iter(&wikitext)
            .chain(ADSENSEUNIT_MODULE_REGEX.find_iter(&wikitext))
            .collect::<Vec<_>>();
        matches.sort_by_key(|matched| matched.start());
        for matched in matches {
            if literal_regions.contains(matched.start()) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(&compat_html.push_block_html("<p>\n\n</p>".to_owned()));
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    pub(super) async fn expand_secondary_runtime_modules(
        ctx: &ServiceContext<'_>,
        mut wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        options: SecondaryRuntimeModuleExpansionOptions<'_>,
        compat_text: &mut CompatTextFragments,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        let make_error =
            || Error::new("failed to perform render operation", ErrorType::Render);
        wikitext = Self::expand_www_currency_convert_system_module(
            wikitext, page_info, settings,
        );
        wikitext = Self::expand_www_special_system_modules(
            ctx,
            wikitext,
            page_info,
            settings,
            options.viewer_user_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = platform_navigation::expand(ctx, wikitext, settings)
            .await
            .or_raise(make_error)?;
        wikitext = platform_directory::expand(
            ctx,
            wikitext,
            settings,
            page_info,
            options.viewer_user_id,
            options.url,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = {
            let _stage = StageGuard::new(options.trace, CorpusRenderStage::CountPages);
            Self::expand_count_pages(
                ctx,
                wikitext,
                page_info,
                settings,
                CountPagesExpansionOptions {
                    current_site_id: options.current_site_id,
                    current_page_id: options.current_page_id,
                },
                compat_text,
                compat_html,
            )
            .await
            .or_raise(make_error)?
        };
        wikitext = Self::expand_join_modules_for_view(
            ctx,
            wikitext,
            settings,
            options.current_site_id,
            options.viewer_user_id,
            options.page_preview,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_membership_apply_modules(
            ctx,
            wikitext,
            settings,
            options.current_site_id,
            options.viewer_user_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_members_modules_with_directory(
            ctx,
            wikitext,
            settings,
            options.current_site_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_list_users_modules(
            ctx,
            wikitext,
            settings,
            options.viewer_user_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_list_drafts_modules(
            ctx,
            wikitext,
            settings,
            options.current_site_id,
            options.viewer_user_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = expand_site_changes_modules(
            ctx,
            wikitext,
            settings,
            options.current_site_id,
            options.viewer_user_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_forum_mini_modules(
            ctx,
            wikitext,
            settings,
            options.current_site_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_forum_modules(
            ctx,
            wikitext,
            settings,
            options.current_site_id,
            options.current_page_id,
            options.viewer_user_id,
            options.url,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = expand_file_modules(
            ctx,
            wikitext,
            page_info,
            settings,
            options.current_site_id,
            options.current_page_id,
            options.viewer_user_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        let search_all_accessible_name = localized_runtime_module_label(
            ctx,
            page_info.language.as_ref(),
            "wiki-page-module-search-all-query",
            "Search all Wikis",
        );
        wikitext = expand_search_feed_modules(
            wikitext,
            settings,
            options.url,
            &search_all_accessible_name,
            compat_html,
        );
        wikitext = Self::expand_simpletodo_modules(wikitext, settings, compat_html);
        wikitext = Self::expand_send_invitations_modules(wikitext, settings, compat_html);
        wikitext = Self::expand_membership_email_invitation_modules(
            ctx,
            wikitext,
            settings,
            options.viewer_user_id,
            options.url,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_static_account_modules(wikitext, settings, compat_html);
        wikitext = expand_site_utility_modules(
            ctx,
            wikitext,
            settings,
            options.current_site_id,
            options.viewer_user_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_membership_by_password_modules(
            ctx,
            wikitext,
            settings,
            options.current_site_id,
            options.viewer_user_id,
            compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_ad_modules(wikitext, settings, compat_html);
        if PAGECALENDAR_MODULE_REGEX.is_match(&wikitext) {
            wikitext = {
                let _stage =
                    StageGuard::new(options.trace, CorpusRenderStage::PageCalendar);
                Self::expand_page_calendar_modules(
                    ctx,
                    wikitext,
                    page_info,
                    settings,
                    PageCalendarExpansionOptions {
                        current_site_id: options.current_site_id,
                        current_page_id: options.current_page_id,
                        url: options.url,
                    },
                    compat_html,
                )
                .await
                .or_raise(make_error)?
            };
        }
        wikitext = {
            let _stage = StageGuard::new(options.trace, CorpusRenderStage::RatedPages);
            Self::expand_rated_pages_modules(
                ctx,
                wikitext,
                settings,
                options.current_site_id,
                options.viewer_user_id,
                compat_html,
            )
            .await
            .or_raise(make_error)?
        };
        wikitext = {
            let _stage = StageGuard::new(options.trace, CorpusRenderStage::TagCloud);
            Self::expand_tag_cloud_modules(
                ctx,
                wikitext,
                page_info,
                settings,
                TagCloudExpansionOptions {
                    current_site_id: options.current_site_id,
                    current_page_id: options.current_page_id,
                },
                compat_text,
                compat_html,
            )
            .await
            .or_raise(make_error)?
        };
        Ok(wikitext)
    }
}

fn wikidot_scope_head_is(source: &str, start: usize, expected: &str) -> bool {
    let Some(tail) = source.get(start + 2..) else {
        return false;
    };
    let Some(end) = tail.find("]]") else {
        return false;
    };
    tail[..end].trim().eq_ignore_ascii_case(expected)
}

#[cfg(test)]
mod theme_previewer_no_ui_tests {
    use super::has_theme_previewer_no_ui;

    #[test]
    fn recognizes_only_executable_no_ui_invocations() {
        assert!(has_theme_previewer_no_ui(
            "[[module ThemePreviewer noUi=\"true\"]]"
        ));
        assert!(!has_theme_previewer_no_ui("[[module ThemePreviewer]]"));
        assert!(!has_theme_previewer_no_ui(
            "[[module ThemePreviewer noUi=\"true\" foo=\"bar\"]]"
        ));
        for source in [
            "[[module ThemePreviewer NOUI=\"true\"]]",
            "[[module ThemePreviewer noUi=\"TRUE\"]]",
            "[[module ThemePreviewer noUi='true']]",
            "[[module ThemePreviewer noUi=true]]",
        ] {
            assert!(!has_theme_previewer_no_ui(source), "{source}");
        }
        assert!(!has_theme_previewer_no_ui(
            "[[code]][[module ThemePreviewer noUi=\"true\"]][[/code]]"
        ));
        assert!(!has_theme_previewer_no_ui(
            "<!-- [[module ThemePreviewer noUi=\"true\"]] -->"
        ));
    }
}

#[cfg(test)]
mod runtime_module_residual_tests {
    use std::borrow::Cow;

    use super::{REDIRECT_MISSING_DESTINATION_HTML, RenderService};
    use crate::services::render::compat::CompatHtmlFragments;
    use crate::services::render::compat::text_fragments::CompatTextFragments;
    use ftml::data::{PageInfo, ScoreValue};
    use ftml::layout::Layout;
    use ftml::render::{Render, html::HtmlRender};
    use ftml::settings::{WikitextMode, WikitextSettings};

    fn render_finalized(source: &str, page_preview: bool) -> String {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let page_info = PageInfo {
            page: Cow::Borrowed("page"),
            category: None,
            site: Cow::Borrowed("site"),
            title: Cow::Borrowed("Page"),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        };
        let mut compat_text = CompatTextFragments::new(source);
        let mut compat_html = CompatHtmlFragments::new(source);
        let mut protected = RenderService::finalize_runtime_module_residuals(
            source.to_owned(),
            &settings,
            page_preview,
            &mut compat_text,
            &mut compat_html,
        );
        ftml::preprocess_for_layout(&mut protected, settings.layout);
        let tokens = ftml::tokenize(&protected);
        let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
        assert!(errors.is_empty(), "{errors:#?}");
        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
        let rendered = compat_html.restore(&rendered);
        compat_text.restore(&rendered)
    }

    #[test]
    fn redirect_preview_renders_destination_notice_and_empty_error() {
        // Live PagePreview observations (sandbox-for-codex, anonymous):
        // a double-quoted non-empty destination renders the redirect
        // notice, while an empty destination renders the same
        // missing-destination error as an absent one. Single-quoted saved
        // destinations use the same missing-destination error.
        let notice =
            render_finalized("[[module Redirect destination=\"start\"]]\n", true);
        assert!(
            notice.contains(concat!(
                "<div class=\"error-block\">\n",
                "\tThis is the Redirect module that redirects the browser ",
                "directly to the &quot;start&quot; page.\n",
                "</div>",
            )),
            "preview must render the live redirect notice:\n{notice}",
        );
        let url_notice = render_finalized(
            "[[module Redirect destination=\"http://example.test/target\"]]\n",
            true,
        );
        assert!(
            url_notice.contains("&quot;http://example.test/target&quot; page."),
            "preview must echo URL destinations as text:\n{url_notice}",
        );
        let empty = render_finalized("[[module Redirect destination=\"\"]]\n", true);
        assert!(
            empty.contains(REDIRECT_MISSING_DESTINATION_HTML),
            "preview must treat an empty destination as unspecified:\n{empty}",
        );
        let saved =
            render_finalized("[[module Redirect destination=\"start\"]]\n", false);
        assert!(
            saved.contains("[[module Redirect destination=&quot;start&quot;]]"),
            "saved renders keep the unobserved shape literal:\n{saved}",
        );
        let saved_single_quoted = render_finalized(
            "[[module Redirect destination='http://example.test/target']]\n",
            false,
        );
        assert!(
            saved_single_quoted.contains(REDIRECT_MISSING_DESTINATION_HTML),
            "saved single-quoted destinations must use the missing-destination error:\n{saved_single_quoted}",
        );
        let preview_single_quoted = render_finalized(
            "[[module Redirect destination='http://example.test/target']]\n",
            true,
        );
        assert!(
            preview_single_quoted.contains(
                "[[module Redirect destination=&#39;http://example.test/target&#39;]]"
            ),
            "preview single-quoted destinations must remain literal:\n{preview_single_quoted}",
        );
    }

    #[test]
    fn finalizes_only_deepwell_owned_residual_modules() {
        let source = concat!(
            "[[module Redirect destination=\"target\"]]\n",
            "[[module NewPage button=\"over-budget\"]]\n",
            "[[module PagesByTag tag=\"a\" limit=\"5\"]]\n",
            "[[module LoginStatus]]\n",
            "[[module LoginStatus foo=\"bar\"]]\n",
            "[[module UnknownOracleModule]]\n",
            "@@[[module NewPage button=\"literal\"]]@@",
        );
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let page_info = PageInfo {
            page: Cow::Borrowed("page"),
            category: None,
            site: Cow::Borrowed("site"),
            title: Cow::Borrowed("Page"),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        };
        let mut compat_text = CompatTextFragments::new(source);
        let mut compat_html = CompatHtmlFragments::new(source);
        let mut protected = RenderService::finalize_runtime_module_residuals(
            source.to_owned(),
            &settings,
            false,
            &mut compat_text,
            &mut compat_html,
        );
        ftml::preprocess_for_layout(&mut protected, settings.layout);
        let tokens = ftml::tokenize(&protected);
        let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
        assert!(errors.is_empty(), "{errors:#?}");
        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
        let rendered = compat_html.restore(&rendered);
        let rendered = compat_text.restore(&rendered);

        for literal in [
            "[[module Redirect destination=&quot;target&quot;]]",
            "[[module NewPage button=&quot;over-budget&quot;]]",
            "[[module PagesByTag tag=&quot;a&quot; limit=&quot;5&quot;]]",
        ] {
            assert!(
                rendered.contains(literal),
                "missing {literal:?}: {rendered}"
            );
        }
        assert!(rendered.contains(concat!(
            r#"<div class="error-block">[[module <em>LoginStatus</em>]] No such module, please "#,
            r#"<a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a>"#,
            " and fix this page.</div>",
        )));
        assert!(rendered.contains(
            r#"[[module <em>LoginStatus</em>]] No such module, please <a href="https://www.wikidot.com/doc:modules""#,
        ));
        assert!(rendered.contains(
            r#"[[module <em>UnknownOracleModule</em>]] No such module, please <a href="https://www.wikidot.com/doc:modules""#,
        ));
        assert!(
            rendered.contains("[[module NewPage button=&quot;literal&quot;]]"),
            "{rendered}",
        );
    }

    #[test]
    fn watchers_argument_bearing_preview_stays_literal_without_watcher_list() {
        // Retained #1032 contract: only the bare opener renders the live
        // empty Watchers block. The documented noActions attribute and
        // unknown attributes have no observed output, so the static preview
        // expansion leaves them byte-literal (later unknown-module) instead
        // of fabricating a watcher list or collapsing to the empty block.
        // Static expansion is actor- and page-free by construction.
        fn preview_static(source: &str) -> String {
            let settings =
                WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
            let mut compat_html = CompatHtmlFragments::new(source);
            let expanded = RenderService::expand_static_account_modules(
                source.to_owned(),
                &settings,
                &mut compat_html,
            );
            compat_html.restore(&expanded)
        }

        let bare = preview_static("[[module Watchers]]");
        assert!(
            bare.contains("<p>\n\n\n\n\n</p>"),
            "bare Watchers must keep the live empty block boundary:\n{bare}",
        );
        assert!(
            !bare.contains("[[module Watchers"),
            "bare Watchers must be consumed, not leaked:\n{bare}",
        );

        for source in [
            "[[module Watchers noActions=\"true\"]]",
            "[[module Watchers foo=\"bar\"]]",
        ] {
            let rendered = preview_static(source);
            assert_eq!(rendered, source, "{source}");
            assert!(
                !rendered.contains("<p>\n\n\n\n\n</p>"),
                "{source} must not collapse to the empty block:\n{rendered}",
            );
        }
    }

    #[test]
    fn native_blank_theme_previewer_has_no_visible_body_and_preserves_literal_regions() {
        assert!(super::has_theme_previewer_blank(
            r#"[[module themepreviewer noUi="true"theme_url=" "]]"#
        ));
        assert!(super::has_theme_previewer_blank(
            r#"[[module ThemePreviewer noUi="true" theme_url=" "]]"#
        ));
        for inactive in [
            r#"[[code]][[module ThemePreviewer noUi="true" theme_url=" "]][[/code]]"#,
            r#"[!-- [[module ThemePreviewer noUi="true" theme_url=" "]] --]"#,
            r#"[[module ThemePreviewer noUi="true" theme_url="https://example.invalid/theme.css"]]"#,
            r#"[[module ThemePreviewer noUi="true" theme_url=" " other="true"]]"#,
        ] {
            assert!(!super::has_theme_previewer_blank(inactive));
        }

        fn expand(source: &str) -> String {
            let settings =
                WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
            let mut fragments = CompatHtmlFragments::new(source);
            let output = RenderService::expand_static_account_modules(
                source.to_owned(),
                &settings,
                &mut fragments,
            );
            fragments.restore(&output)
        }
        for source in [
            "[[module themepreviewer noUi=\"true\"theme_url=\" \"]]",
            "[[module ThemePreviewer noUi=\"true\" theme_url=\" \"]]",
        ] {
            assert_eq!(
                expand(&format!("before\n{source}\nafter")),
                "before\n\nafter"
            );
            for literal in [
                format!("[[code]]{source}[[/code]]"),
                format!("[!-- {source} --]"),
            ] {
                assert_eq!(expand(&literal), literal);
            }
        }
        for source in [
            "[[module ThemePreviewer noUi=\"true\" theme_url=\"https://example.test/style.css\"]]",
            "[[module ThemePreviewer noUi=\"true\" theme_url=\" \" foo=\"bar\"]]",
            "[[module ThemePreviewer noUi=\"false\" theme_url=\" \"]]",
        ] {
            assert_eq!(expand(source), source);
        }
    }

    #[test]
    fn dashboard_and_themepreviewer_argument_bearing_preview_stays_literal() {
        // Retained #1038 contract: only the bare openers render the live
        // Dashboard/ThemePreviewer errors. The documented ThemePreviewer
        // noUi attribute and unknown attributes have no observed output, so
        // the static preview expansion leaves them byte-literal (later
        // unknown-module) instead of rendering an error branch or fetching
        // an author-supplied theme URL. Static expansion is actor- and
        // page-free by construction.
        fn preview_static(source: &str) -> String {
            let settings =
                WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
            let mut compat_html = CompatHtmlFragments::new(source);
            let expanded = RenderService::expand_static_account_modules(
                source.to_owned(),
                &settings,
                &mut compat_html,
            );
            compat_html.restore(&expanded)
        }

        let dashboard = preview_static("[[module Dashboard]]");
        assert!(
            dashboard.contains("Not allowed. Error."),
            "bare Dashboard must keep the live error boundary:\n{dashboard}",
        );
        assert!(
            !dashboard.contains("[[module Dashboard"),
            "bare Dashboard must be consumed, not leaked:\n{dashboard}",
        );

        let previewer = preview_static("[[module ThemePreviewer]]");
        assert!(
            previewer.contains(
                "Preview mode error: please contact Wikidot.com for a better error message",
            ),
            "bare ThemePreviewer must keep the live error boundary:\n{previewer}",
        );
        assert!(
            !previewer.contains("[[module ThemePreviewer"),
            "bare ThemePreviewer must be consumed, not leaked:\n{previewer}",
        );

        for source in [
            "[[module Dashboard foo=\"bar\"]]",
            "[[module ThemePreviewer noUi=\"true\"]]",
            "[[module ThemePreviewer foo=\"bar\"]]",
        ] {
            let rendered = preview_static(source);
            assert_eq!(rendered, source, "{source}");
        }
    }

    #[test]
    fn www_counter_groups_thousands_with_spaces_like_live() {
        // Retained #1508 live www front-special stats: large counters use
        // space-separated thousands groups. Pin the general grouping rule
        // (multi-group positives plus sub-thousand, zero, and clamped
        // negative controls) without replaying any captured population.
        for (value, expected) in [
            (106_410_870_i64, "106 410 870"),
            (31_444_i64, "31 444"),
            (10_514_050_i64, "10 514 050"),
            (749_i64, "749"),
            (1_000_i64, "1 000"),
            (999_i64, "999"),
            (0_i64, "0"),
            (-42_i64, "0"),
        ] {
            assert_eq!(
                RenderService::format_www_counter(value),
                expected,
                "{value}"
            );
        }
    }
}

#[cfg(test)]
mod simpletodo_security_tests {
    use super::render_simpletodo_module;

    #[test]
    fn valid_simpletodo_shell_contains_no_active_page_content() {
        let html = render_simpletodo_module(r#" id="fixture""#, 0);

        for forbidden in [
            "<script",
            "http://www.wikidot.com/common--javascript/yahooui/animation-min.js",
            "javascript:",
            " onclick=",
            " onload=",
            " onerror=",
        ] {
            assert!(!html.contains(forbidden), "found {forbidden:?}: {html}");
        }
        assert!(html.contains(r#"<div class="simpletodo-box" id="simpletodo_0">"#));
        assert!(html.contains(r#"<div class="label">fixture</div>"#));
        assert!(
            html.contains(r#"<span id="simpletodo-data-edit-permission">false</span>"#)
        );
        assert_eq!(html.matches(r#"aria-disabled="true""#).count(), 2);
    }
}

#[cfg(test)]
mod localized_module_accessible_name_tests {
    use std::path::PathBuf;

    use super::{
        NewPageTemplateRendering, localized_runtime_module_label_from_localizations,
        render_new_page_module,
    };
    use crate::locales::Localizations;
    use crate::services::render::compat::CompatHtmlFragments;
    use crate::services::render::search_feed::expand_search_feed_modules;
    use crate::services::render::url_arguments::UrlArguments;
    use ftml::layout::Layout;
    use ftml::settings::{WikitextMode, WikitextSettings};

    #[tokio::test]
    async fn japanese_accessible_names_flow_from_fluent_catalogs_into_module_renderers() {
        let locale_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../locales");
        let localizations = Localizations::open(locale_root).await.unwrap();
        let new_page_name = localized_runtime_module_label_from_localizations(
            &localizations,
            "ja",
            "wiki-page-module-new-page-name",
            "Name of the new page",
        );
        let search_all_name = localized_runtime_module_label_from_localizations(
            &localizations,
            "ja",
            "wiki-page-module-search-all-query",
            "Search all Wikis",
        );

        assert_eq!(new_page_name, "新しいページ名");
        assert_eq!(search_all_name, "すべてのWikiを検索");
        let new_page_html =
            render_new_page_module("", NewPageTemplateRendering::None, &new_page_name);
        assert!(new_page_html.contains(r#"aria-label="新しいページ名""#));

        let source = "[[module SearchAll]]";
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut compat_html = CompatHtmlFragments::new(source);
        let expanded = expand_search_feed_modules(
            source.to_owned(),
            &settings,
            UrlArguments::default(),
            &search_all_name,
            &mut compat_html,
        );
        let search_all_html = compat_html.restore(&expanded);
        assert!(search_all_html.contains(r#"aria-label="すべてのWikiを検索""#));
    }
}
