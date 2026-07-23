/*
 * services/settings/service.rs
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

use super::prelude::*;
use crate::license::WikidotLicense;
use crate::services::forum::GetForumCategory;
use crate::services::{
    CategoryService, ForumService, PageRevisionService, PageService, SiteService,
};
use crate::types::parse_layout;
use ftml::layout::Layout;
use std::borrow::Cow;

#[derive(Debug)]
pub struct SettingsService;

impl SettingsService {
    pub async fn get_page_discussion_settings(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_id: i64,
    ) -> Result<PageDiscussionSettings> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to get page discussion settings for site ID {site_id}, category ID {category_id}"
                ),
                ErrorType::SiteSettings,
            )
        };
        let category = CategoryService::get(ctx, site_id, Reference::Id(category_id))
            .await
            .or_raise(make_error)?;
        let default_category = if category.slug == "_default" {
            None
        } else {
            CategoryService::get_optional(
                ctx,
                site_id,
                Reference::Slug(Cow::Borrowed("_default")),
            )
            .await
            .or_raise(make_error)?
        };

        Ok(PageDiscussionSettings {
            enabled: category
                .per_page_discussion
                .or_else(|| {
                    default_category
                        .as_ref()
                        .and_then(|value| value.per_page_discussion)
                })
                .unwrap_or(false),
        })
    }

    pub async fn get_page_rating_settings(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_id: i64,
    ) -> Result<PageRatingSettings> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to get page rating settings for site ID {site_id}, category ID {category_id}"
                ),
                ErrorType::SiteSettings,
            )
        };
        let category = CategoryService::get(ctx, site_id, Reference::Id(category_id))
            .await
            .or_raise(make_error)?;
        let default_category = if category.slug == "_default" {
            None
        } else {
            CategoryService::get_optional(
                ctx,
                site_id,
                Reference::Slug(Cow::Borrowed("_default")),
            )
            .await
            .or_raise(make_error)?
        };
        let fallback = PageRatingSettings::default();
        let permission = category
            .rating_permission
            .as_deref()
            .or_else(|| {
                default_category
                    .as_ref()
                    .and_then(|value| value.rating_permission.as_deref())
            })
            .and_then(PageRatingPermission::from_storage)
            .unwrap_or(fallback.permission);
        let visibility = category
            .rating_visibility
            .as_deref()
            .or_else(|| {
                default_category
                    .as_ref()
                    .and_then(|value| value.rating_visibility.as_deref())
            })
            .and_then(PageRatingVisibility::from_storage)
            .unwrap_or(fallback.visibility);
        let rating_type = category
            .rating_type
            .as_deref()
            .or_else(|| {
                default_category
                    .as_ref()
                    .and_then(|value| value.rating_type.as_deref())
            })
            .and_then(PageRatingType::from_storage)
            .unwrap_or(fallback.rating_type);

        Ok(PageRatingSettings {
            enabled: category
                .rating_enabled
                .or_else(|| {
                    default_category
                        .as_ref()
                        .and_then(|value| value.rating_enabled)
                })
                .unwrap_or(fallback.enabled),
            permission,
            visibility,
            rating_type,
        })
    }

    /// Get the effective license for a page category.
    ///
    /// A category override wins when present. Otherwise the `_default`
    /// category is inherited, with the site license as the legacy fallback.
    pub async fn get_license(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_id: Option<i64>,
    ) -> Result<WikidotLicense> {
        let make_error = || {
            Error::new(
                match category_id {
                    Some(category_id) => format!(
                        "failed to get license for site ID {}, category ID {}",
                        site_id, category_id,
                    ),
                    None => format!(
                        "failed to get license for site ID {}, no category",
                        site_id,
                    ),
                },
                ErrorType::SiteSettings,
            )
        };

        if let Some(category_id) = category_id {
            let category = CategoryService::get(ctx, site_id, Reference::Id(category_id))
                .await
                .or_raise(make_error)?;
            if let Some(license) = category.license.as_deref() {
                return WikidotLicense::from_storage(
                    license,
                    category.license_other.as_deref(),
                )
                .or_raise(make_error);
            }
        }

        let default_category = CategoryService::get_optional(
            ctx,
            site_id,
            Reference::Slug(Cow::Borrowed("_default")),
        )
        .await
        .or_raise(make_error)?;
        if let Some(category) = default_category
            && let Some(license) = category.license.as_deref()
        {
            return WikidotLicense::from_storage(
                license,
                category.license_other.as_deref(),
            )
            .or_raise(make_error);
        }

        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;
        Ok(WikidotLicense::Standard(site.license))
    }

    /// Get the layout associated with this page.
    ///
    /// If this page has a specific layout override,
    /// then that is returned. Otherwise, the layout
    /// associated with the site is used.
    ///
    /// If no page ID is specified, then searching
    /// starts with site layout settings.
    pub async fn get_layout(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: Option<i64>,
    ) -> Result<Layout> {
        let make_error = || {
            Error::new(
                match page_id {
                    Some(page_id) => format!(
                        "failed to get layout for site ID {}, page ID {}",
                        site_id, page_id,
                    ),
                    None => {
                        format!("failed to get layout for site ID {}, no page", site_id)
                    }
                },
                ErrorType::SiteSettings,
            )
        };

        let mut page_from_wikidot = false;
        if let Some(page_id) = page_id {
            debug!("Getting layout for site ID {site_id} page ID {page_id}");
            let page = PageService::get_direct(ctx, page_id, true)
                .await
                .or_raise(make_error)?;

            if let Some(layout) = page.layout {
                debug!("Found page-level layout override: {layout}");
                return parse_layout(&layout).or_raise(make_error);
            }

            page_from_wikidot = page.from_wikidot;

            let category_id = page.page_category_id;
            debug!("Getting layout for page category ID {category_id}");
            let category = CategoryService::get(ctx, site_id, Reference::Id(category_id))
                .await
                .or_raise(make_error)?;

            if let Some(layout) = category.layout {
                debug!("Found category-level layout override: {layout}");
                return parse_layout(&layout).or_raise(make_error);
            }
        }

        debug!("Getting layout for site ID {site_id}");
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;

        if let Some(layout) = site.layout {
            debug!("Found site-level layout override: {layout}");
            return parse_layout(&layout).or_raise(make_error);
        }

        if page_from_wikidot {
            debug!("Using Wikidot layout for imported page provenance");
        } else {
            debug!("Using platform-level layout");
        }
        Ok(default_page_layout_for_provenance(
            page_from_wikidot,
            ctx.config().default_page_layout,
        ))
    }

    /// Get the navigation pages for this page category.
    ///
    /// If this category has nav page overrides, then those
    /// are returned. Otherwise, the respective navigation
    /// pages for the site is used.
    ///
    /// If no category ID is specified, then searching
    /// starts with site nav page settings.
    ///
    /// Note that empty strings have a special meaning,
    /// specifying that this navigation element is not included.
    pub async fn get_nav_page_slugs(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_id: Option<i64>,
    ) -> Result<NavigationPageSlugs> {
        let make_error = || {
            Error::new(
                match category_id {
                    Some(category_id) => format!(
                        "failed to get nav page slugs for site ID {}, category ID {}",
                        site_id, category_id,
                    ),
                    None => format!(
                        "failed to get nav page slugs for site ID {}, no category",
                        site_id,
                    ),
                },
                ErrorType::SiteSettings,
            )
        };

        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;

        let (override_top_bar, override_side_bar) = match category_id {
            None => (None, None),
            Some(category_id) => {
                let category =
                    CategoryService::get(ctx, site_id, Reference::Id(category_id))
                        .await
                        .or_raise(make_error)?;

                (category.top_bar_page, category.side_bar_page)
            }
        };

        Ok(NavigationPageSlugs {
            top_bar_page: override_top_bar.unwrap_or(site.top_bar_page).into(),
            side_bar_page: override_side_bar.unwrap_or(site.side_bar_page).into(),
        })
    }

    /// Get the current page wikitexts for the current navigation pages.
    ///
    /// This is essentially a convenience method for `get_nav_page_slugs()`
    /// to also fetch the page wikitext values as well. It is used in
    /// `RenderService` to produce the compiled nav HTML columns for storage.
    pub async fn get_nav_page_wikitext(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_id: Option<i64>,
    ) -> Result<NavigationPageWikitext> {
        let make_error = || {
            Error::new(
                match category_id {
                    Some(category_id) => format!(
                        "failed to get nav page wikitext contents for site ID {}, category ID {}",
                        site_id, category_id,
                    ),
                    None => format!(
                        "failed to get nav page wikitext contents for site ID {}, no category",
                        site_id,
                    ),
                },
                ErrorType::SiteSettings,
            )
        };

        let NavigationPageSlugs {
            top_bar_page,
            side_bar_page,
        } = Self::get_nav_page_slugs(ctx, site_id, category_id)
            .await
            .or_raise(make_error)?;

        // Helper function so we can do a clean try_join!
        async fn get_wikitext(
            ctx: &ServiceContext<'_>,
            site_id: i64,
            page: &NavigationPage,
        ) -> Result<Option<String>> {
            let page_slug = match page {
                NavigationPage::Enabled(page_slug) => page_slug,
                NavigationPage::Disabled => return Ok(None),
            };

            PageRevisionService::get_wikitext_optional(
                ctx,
                site_id,
                Reference::Slug(cow!(page_slug)),
            )
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get wikitext for page '{}' in site ID {}",
                        page_slug, site_id,
                    ),
                    ErrorType::Text,
                )
            })
        }

        let (top_bar_wikitext_result, side_bar_wikitext_result) = join!(
            get_wikitext(ctx, site_id, &top_bar_page),
            get_wikitext(ctx, site_id, &side_bar_page),
        );

        let (top_bar_page_wikitext, side_bar_page_wikitext) = raise_multiple!(top_bar_wikitext_result, side_bar_wikitext_result; make_error);

        Ok(NavigationPageWikitext {
            top_bar_page_wikitext,
            side_bar_page_wikitext,
        })
    }

    /// Get the compiled page HTML for the current navigation pages.
    ///
    /// This is use to get nav page contents *only for missing or invalid pages*.
    /// Any pages which exist have their own cached `compiled_xxx_bar_html_hash`
    /// columns which can be used instead.
    pub async fn get_nav_page_html(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_id: Option<i64>,
    ) -> Result<NavigationPageHtml> {
        let make_error = || {
            Error::new(
                match category_id {
                    Some(category_id) => format!(
                        "failed to get nav page HTML for site ID {}, category ID {}",
                        site_id, category_id,
                    ),
                    None => format!(
                        "failed to get nav page HTML for site ID {}, no category",
                        site_id,
                    ),
                },
                ErrorType::SiteSettings,
            )
        };

        let NavigationPageSlugs {
            top_bar_page,
            side_bar_page,
        } = Self::get_nav_page_slugs(ctx, site_id, category_id)
            .await
            .or_raise(make_error)?;

        // Helper function, like above
        async fn get_html(
            ctx: &ServiceContext<'_>,
            site_id: i64,
            page: &NavigationPage,
        ) -> Result<Option<String>> {
            let page_slug = match page {
                NavigationPage::Enabled(page_slug) => page_slug,
                NavigationPage::Disabled => return Ok(None),
            };

            PageRevisionService::get_compiled_html_optional(
                ctx,
                site_id,
                Reference::Slug(cow!(page_slug)),
            )
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get HTML for page '{}' in site ID {}",
                        page_slug, site_id,
                    ),
                    ErrorType::Text,
                )
            })
        }

        let (top_bar_html_result, side_bar_html_result) = join!(
            get_html(ctx, site_id, &top_bar_page),
            get_html(ctx, site_id, &side_bar_page),
        );

        let (compiled_top_bar_html, compiled_side_bar_html) =
            raise_multiple!(top_bar_html_result, side_bar_html_result; make_error);

        Ok(NavigationPageHtml {
            compiled_top_bar_html,
            compiled_side_bar_html,
        })
    }

    /// Gets forum settings, combining site defaults and category overrides.
    ///
    /// Category settings (if specified) override site-level defaults.
    #[allow(dead_code)] // TODO
    pub async fn get_forum_settings(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        forum_category_id: Option<i64>,
    ) -> Result<ForumStructureSettings> {
        let make_error = || {
            Error::new(
                match forum_category_id {
                    Some(forum_category_id) => format!(
                        "failed to get forum settings for site ID {}, category ID {}",
                        site_id, forum_category_id,
                    ),
                    None => format!(
                        "failed to get forum settings for site ID {}, no category",
                        site_id,
                    ),
                },
                ErrorType::Forum,
            )
        };

        let defaults = SiteService::get_forum_settings(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;

        let category = match forum_category_id {
            Some(forum_category_id) => Some(
                ForumService::get_category(
                    ctx,
                    GetForumCategory {
                        site_id,
                        forum_category_id,
                        include_deleted: false,
                    },
                )
                .await
                .or_raise(make_error)?,
            ),
            None => None,
        };

        Ok(ForumStructureSettings {
            max_nest_level: category
                .as_ref()
                .and_then(|category| category.max_nest_level)
                .unwrap_or(defaults.max_nest_level),
            per_page_discussion: category
                .as_ref()
                .and_then(|category| category.per_page_discussion)
                .unwrap_or(defaults.per_page_discussion),
        })
    }

    #[allow(dead_code)] // TODO
    #[inline]
    pub async fn get_forum_max_nest_level(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        forum_category_id: Option<i64>,
    ) -> Result<i16> {
        let settings = Self::get_forum_settings(ctx, site_id, forum_category_id).await?;
        Ok(settings.max_nest_level)
    }

    #[allow(dead_code)] // TODO
    #[inline]
    pub async fn get_forum_per_page_discussion(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        forum_category_id: Option<i64>,
    ) -> Result<bool> {
        let settings = Self::get_forum_settings(ctx, site_id, forum_category_id).await?;
        Ok(settings.per_page_discussion)
    }
}

fn default_page_layout_for_provenance(
    from_wikidot: bool,
    platform_default: Layout,
) -> Layout {
    if from_wikidot {
        Layout::Wikidot
    } else {
        platform_default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imported_page_provenance_selects_wikidot_layout_over_platform_default() {
        assert_eq!(
            default_page_layout_for_provenance(true, Layout::Wikijump),
            Layout::Wikidot,
        );
    }

    #[test]
    fn local_page_provenance_keeps_platform_default() {
        assert_eq!(
            default_page_layout_for_provenance(false, Layout::Wikijump),
            Layout::Wikijump,
        );
    }
}
