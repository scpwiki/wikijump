//! Runtime expansion for Wikidot membership modules.

#[cfg(test)]
use super::membership_apply_action_count;
use super::{
    CompatHtmlFragments, GetSiteMember, LiteralRegionIndex,
    MEMBERSHIP_APPLY_ALREADY_APPLIED_HTML, MEMBERSHIP_APPLY_ANONYMOUS_HTML,
    MEMBERSHIP_APPLY_DISABLED_HTML, MEMBERSHIP_APPLY_FORM_HTML,
    MEMBERSHIP_APPLY_MEMBER_HTML, MEMBERSHIP_BY_PASSWORD_ANONYMOUS_HTML,
    MEMBERSHIP_BY_PASSWORD_DISABLED_HTML, MEMBERSHIP_BY_PASSWORD_FORM_HTML,
    MEMBERSHIP_BY_PASSWORD_MEMBER_HTML, MEMBERSHIP_EMAIL_INVITATION_MISSING_HTML,
    MEMBERSHIPAPPLY_MODULE_REGEX, MEMBERSHIPBYPASSWORD_MODULE_REGEX,
    MembershipByPasswordResultCache, MembershipService, Reference, RelationObject,
    RelationReference, RelationService, RelationType, RenderService, Result,
    STATIC_ACCOUNT_MODULE_REGEX, ServiceContext, SiteService, UrlArguments, UserService,
    WikitextSettings, render_membership_email_invitation_valid,
};

impl RenderService {
    pub(super) async fn expand_membership_apply_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        viewer_user_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax
            || !MEMBERSHIPAPPLY_MODULE_REGEX.is_match(&wikitext)
        {
            return Ok(wikitext);
        }
        let Some(site_id) = current_site_id else {
            return Ok(wikitext);
        };
        let site = SiteService::get(ctx, Reference::Id(site_id)).await?;
        let rendered = if !site.membership_by_application {
            MEMBERSHIP_APPLY_DISABLED_HTML
        } else if let Some(user_id) = viewer_user_id {
            if RelationService::site_member_exists(
                ctx,
                GetSiteMember { site_id, user_id },
            )
            .await?
            {
                MEMBERSHIP_APPLY_MEMBER_HTML
            } else if RelationService::exists(
                ctx,
                RelationReference::Relationship {
                    relation_type: RelationType::SiteApplication,
                    dest: RelationObject::Site(site_id),
                    from: RelationObject::User(user_id),
                },
            )
            .await?
            {
                MEMBERSHIP_APPLY_ALREADY_APPLIED_HTML
            } else {
                MEMBERSHIP_APPLY_FORM_HTML
            }
        } else {
            MEMBERSHIP_APPLY_ANONYMOUS_HTML
        };
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in MEMBERSHIPAPPLY_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a MembershipApply capture always has a complete match");
            if literal_regions.contains(matched.start())
                || captures
                    .name("head")
                    .is_some_and(|head| !head.as_str().trim().is_empty())
            {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(&compat_html.push_block_html(rendered.to_owned()));
            cursor = matched.end();
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }

    pub(super) async fn expand_membership_email_invitation_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        viewer_user_id: Option<i64>,
        url: UrlArguments<'_>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax
            || !STATIC_ACCOUNT_MODULE_REGEX.is_match(&wikitext)
        {
            return Ok(wikitext);
        }
        let hash = url
            .path_arguments
            .iter()
            .rfind(|argument| argument.name.eq_ignore_ascii_case("hash"))
            .and_then(|argument| argument.value.as_deref())
            .filter(|value| !value.is_empty());
        let invitation = match hash {
            Some(hash) => MembershipService::resolve_email_invitation(ctx, hash).await?,
            None => None,
        };
        let viewer = match viewer_user_id {
            Some(user_id) => UserService::get(ctx, Reference::Id(user_id))
                .await?
                .into_public_identity()
                .map(|identity| {
                    (
                        user_id,
                        identity.user_name.into_owned(),
                        identity.user_profile_url.into_owned(),
                    )
                }),
            None => None,
        };
        let rendered = match (hash, invitation.as_ref()) {
            (Some(hash), Some(invitation)) => {
                render_membership_email_invitation_valid(invitation, viewer, hash)
            }
            _ => MEMBERSHIP_EMAIL_INVITATION_MISSING_HTML.to_owned(),
        };

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in STATIC_ACCOUNT_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a static account module capture always has a complete match");
            if literal_regions.contains(matched.start())
                || !captures.name("name").is_some_and(|name| {
                    name.as_str()
                        .eq_ignore_ascii_case("MembershipEmailInvitation")
                })
                || captures
                    .name("head")
                    .is_some_and(|head| !head.as_str().trim().is_empty())
            {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(&compat_html.push_block_html(rendered.clone()));
            cursor = matched.end();
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }

    async fn render_membership_by_password_module(
        ctx: &ServiceContext<'_>,
        current_site_id: Option<i64>,
        viewer_user_id: Option<i64>,
    ) -> Result<Option<&'static str>> {
        let Some(current_site_id) = current_site_id else {
            return Ok(None);
        };
        let site = SiteService::get(ctx, Reference::Id(current_site_id)).await?;
        if !site.membership_by_password || site.membership_password_hash.is_none() {
            return Ok(Some(MEMBERSHIP_BY_PASSWORD_DISABLED_HTML));
        }
        let Some(viewer_user_id) = viewer_user_id else {
            return Ok(Some(MEMBERSHIP_BY_PASSWORD_ANONYMOUS_HTML));
        };
        let membership = RelationService::get_optional_site_member(
            ctx,
            GetSiteMember {
                site_id: current_site_id,
                user_id: viewer_user_id,
            },
        )
        .await?;
        Ok(Some(if membership.is_some() {
            MEMBERSHIP_BY_PASSWORD_MEMBER_HTML
        } else {
            MEMBERSHIP_BY_PASSWORD_FORM_HTML
        }))
    }

    pub(super) async fn expand_membership_by_password_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        viewer_user_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax
            || !MEMBERSHIPBYPASSWORD_MODULE_REGEX.is_match(&wikitext)
        {
            return Ok(wikitext);
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        let mut result_cache = MembershipByPasswordResultCache::default();
        for captures in MEMBERSHIPBYPASSWORD_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a MembershipByPassword capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let head = captures.name("head").map_or("", |head| head.as_str());
            if !head.trim().is_empty() {
                continue;
            }
            let rendered = result_cache
                .get_or_init(|| {
                    Self::render_membership_by_password_module(
                        ctx,
                        current_site_id,
                        viewer_user_id,
                    )
                })
                .await?;
            let Some(rendered) = rendered else {
                continue;
            };
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(&compat_html.push_block_html(rendered.to_owned()));
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
mod membership_by_password_tests {
    use std::cell::Cell;

    use super::MembershipByPasswordResultCache;

    #[tokio::test]
    async fn repeated_membership_modules_reuse_the_same_render_result() {
        let loads = Cell::new(0);
        let mut cache = MembershipByPasswordResultCache::default();

        let first = cache
            .get_or_init(|| async {
                loads.set(loads.get() + 1);
                Ok(Some("member"))
            })
            .await
            .expect("the first membership lookup should succeed");
        let second = cache
            .get_or_init(|| async {
                panic!("a cached membership result must not query again");
                #[allow(unreachable_code)]
                Ok(None)
            })
            .await
            .expect("the cached membership result should succeed");

        assert_eq!(first, Some("member"));
        assert_eq!(second, Some("member"));
        assert_eq!(loads.get(), 1);
    }
}

#[cfg(test)]
mod membership_apply_tests {
    use super::membership_apply_action_count;

    #[test]
    fn membership_apply_registry_accepts_only_bare_nonliteral_modules() {
        assert_eq!(
            membership_apply_action_count("[[module MembershipApply]]"),
            1
        );
        assert_eq!(
            membership_apply_action_count("[[module MEMBERSHIPAPPLY]]"),
            1
        );
        assert_eq!(
            membership_apply_action_count("[[module MembershipApply foo=\"bar\"]]"),
            0,
        );
        assert_eq!(
            membership_apply_action_count("[[module MembershipApply limit=\"5\"]]"),
            0,
        );
        assert_eq!(
            membership_apply_action_count("[[code]][[module MembershipApply]][[/code]]"),
            0,
        );
    }
}
