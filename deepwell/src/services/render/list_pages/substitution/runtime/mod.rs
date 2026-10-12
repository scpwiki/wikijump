/*
 * services/render/list_pages/substitution/runtime/mod.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Runtime ListPages row substitution and its focused regressions.

use super::{
    BTreeMap, BTreeSet, CompatHtmlFragments, CompatTextFragments, Cow, FoundPageRow,
    LISTPAGES_VARIABLE_REGEX, ListPagesDataFormDefinition, ListPagesGeneratedSlot,
    ListPagesParentDisplay, ListPagesRuntimeDisplay, ListPagesRuntimeTextRange,
    ListPagesSnapshotDisplay, RenderService, WikidotCompatLinkTitleMap,
    WikidotUserDisplay, collect_list_pages_html_body_ranges, escape_html_text,
    format_list_pages_created_at, format_list_pages_rating, ftml,
    is_list_pages_hidden_tag, is_list_pages_visible_tag, list_pages_created_by_slug,
    list_pages_first_paragraph, list_pages_plain_text, list_pages_preview,
    list_pages_preview_length, list_pages_tag_target_prefix,
    list_pages_variable_capture_has_unknown_name, list_pages_variable_capture_is_valid,
    list_pages_variable_starts_triple_link_target, neutralize_authored_markers,
    protect_list_pages_content_insertion, protect_list_pages_generated_html,
    push_list_pages_rendered_fragment, push_list_pages_rendered_fragment_with_mode,
    render_list_pages_form_wiki_value, render_list_pages_linked_title,
    render_list_pages_snapshot_user, render_list_pages_snapshot_wikidot_user,
    render_list_pages_tags, render_list_pages_wikidot_user, sanitize_list_pages_title,
    substitute_list_pages_form_data, substitute_list_pages_form_hint,
    substitute_list_pages_form_label, substitute_list_pages_form_raw,
    wikidot_content_section, wikidot_empty_imported_title_label,
};
#[cfg(test)]
use super::{
    list_pages_rendered_fragment_has_html_block, list_pages_rendered_inline_fragment,
    take_empty_paragraph_restore_scanned_bytes,
};

pub(in crate::services::render) struct ListPagesSubstitutionContext<'a> {
    pub(in crate::services::render) authored_limit: Option<u64>,
    pub(in crate::services::render) ajax_module_response: bool,
    pub(in crate::services::render) page_preview: bool,
    pub(in crate::services::render) site: &'a str,
    pub(in crate::services::render) site_title: &'a str,
    pub(in crate::services::render) category: &'a str,
    pub(in crate::services::render) tag_target: Option<&'a str>,
    pub(in crate::services::render) user_displays: &'a BTreeMap<i64, WikidotUserDisplay>,
    pub(in crate::services::render) snapshot_displays:
        &'a BTreeMap<i64, ListPagesSnapshotDisplay>,
    pub(in crate::services::render) runtime_displays:
        &'a BTreeMap<i64, ListPagesRuntimeDisplay>,
    pub(in crate::services::render) page_wikitext: Option<&'a str>,
    pub(in crate::services::render) page_rendered_content: Option<&'a str>,
    pub(in crate::services::render) page_rendered_summary: Option<&'a str>,
    pub(in crate::services::render) page_rendered_summary_is_block: bool,
    pub(in crate::services::render) default_summary_first_paragraph: bool,
    pub(in crate::services::render) fallback_link_titles:
        Option<&'a WikidotCompatLinkTitleMap>,
    pub(in crate::services::render) page_rendered_first_paragraph: Option<&'a str>,
    pub(in crate::services::render) page_compiled_body_html: Option<&'a str>,
    pub(in crate::services::render) page_wikitext_scalar_count: Option<usize>,
    pub(in crate::services::render) page_parent_fullname: Option<&'a str>,
    pub(in crate::services::render) page_parent_display:
        Option<&'a ListPagesParentDisplay>,
    pub(in crate::services::render) page_child_count: Option<u64>,
    pub(in crate::services::render) page_revision_count: Option<u64>,
    pub(in crate::services::render) data_form_values: &'a BTreeMap<String, String>,
    pub(in crate::services::render) data_form_definition:
        Option<&'a ListPagesDataFormDefinition>,
    pub(in crate::services::render) render_generated_html: bool,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::services::render::list_pages) fn substitute_list_pages_variables_inner(
    template: &str,
    page: &FoundPageRow,
    index: usize,
    total: usize,
    context: &ListPagesSubstitutionContext<'_>,
    compat_html: &mut CompatHtmlFragments,
    compat_text: &mut CompatTextFragments,
    mut tracked_content_fragments: Option<&mut BTreeSet<usize>>,
    mut generated_slots: Option<&mut Vec<ListPagesGeneratedSlot>>,
    mut runtime_text_ranges: Option<&mut Vec<ListPagesRuntimeTextRange>>,
    runtime_title_is_delayed: bool,
) -> String {
    let html_body_ranges = collect_list_pages_html_body_ranges(template);
    let full_slug = page.slug.as_deref().unwrap_or("");
    // Page-query rows already retain Wikidot's normalized full slug, including
    // a non-default category prefix. Reconstructing it would duplicate that prefix.
    let slug = if context.category.is_empty() {
        full_slug
    } else {
        full_slug
            .strip_prefix(context.category)
            .and_then(|slug| slug.strip_prefix(':'))
            .unwrap_or(full_slug)
    };
    let link = format!("http://{}.wikidot.com/{full_slug}", context.site);
    let snapshot = context.snapshot_displays.get(&page.page_id);
    let runtime = context.runtime_displays.get(&page.page_id);
    let title = match snapshot.and_then(|snapshot| snapshot.title_shown.as_deref()) {
        Some("") => wikidot_empty_imported_title_label(full_slug),
        Some(title) => title.to_owned(),
        None => page.title.as_deref().unwrap_or(slug).to_owned(),
    };
    let title = sanitize_list_pages_title(&title);
    let mut title_plain = title.clone();
    if !runtime_title_is_delayed {
        ftml::preproc::typography::substitute_wikidot(&mut title_plain);
    }
    let title_linked = render_list_pages_linked_title(full_slug, &title, compat_text);
    let created_by_snapshot =
        snapshot.and_then(|snapshot| snapshot.created_by_name.as_deref());
    let updated_by_snapshot =
        snapshot.and_then(|snapshot| snapshot.updated_by_name.as_deref());
    let created_by = created_by_snapshot
        .map(str::to_owned)
        .or_else(|| {
            page.created_by.map(|user_id| {
                context
                    .user_displays
                    .get(&user_id)
                    .map(|user| user.name.clone())
                    .unwrap_or_else(|| user_id.to_string())
            })
        })
        .unwrap_or_default();
    let created_by_unix = list_pages_created_by_slug(
        page,
        context.user_displays,
        context.snapshot_displays,
    );
    let created_by_id = if created_by_snapshot.is_some() {
        snapshot
            .and_then(|snapshot| snapshot.created_by_user_id)
            .map(|user_id| user_id.to_string())
            .unwrap_or_default()
    } else {
        page.created_by
            .map(|user_id| user_id.to_string())
            .unwrap_or_default()
    };
    let created_by_linked = created_by_snapshot
        .map(|name| {
            render_list_pages_snapshot_wikidot_user(
                name,
                snapshot.and_then(|snapshot| snapshot.created_by_user_id),
                snapshot.and_then(|snapshot| snapshot.created_by_slug.as_deref()),
            )
        })
        .or_else(|| {
            page.created_by.map(|user_id| {
                render_list_pages_wikidot_user(
                    user_id,
                    context.user_displays.get(&user_id),
                )
            })
        })
        .unwrap_or_default();
    let updated_by = updated_by_snapshot
        .map(str::to_owned)
        .or_else(|| {
            page.updated_by.map(|user_id| {
                context
                    .user_displays
                    .get(&user_id)
                    .map(|user| user.name.clone())
                    .unwrap_or_else(|| user_id.to_string())
            })
        })
        .unwrap_or_default();
    let updated_by_linked = updated_by_snapshot
        .map(|name| {
            render_list_pages_snapshot_wikidot_user(
                name,
                snapshot.and_then(|snapshot| snapshot.updated_by_user_id),
                snapshot.and_then(|snapshot| snapshot.updated_by_slug.as_deref()),
            )
        })
        .or_else(|| {
            page.updated_by.map(|user_id| {
                render_list_pages_wikidot_user(
                    user_id,
                    context.user_displays.get(&user_id),
                )
            })
        })
        .unwrap_or_default();
    let updated_by_slug = if updated_by_snapshot.is_some() {
        snapshot
            .and_then(|snapshot| snapshot.updated_by_slug.clone())
            .unwrap_or_default()
    } else {
        page.updated_by
            .and_then(|user_id| context.user_displays.get(&user_id))
            .and_then(|user| user.slug.clone())
            .unwrap_or_default()
    };
    let updated_by_id = if updated_by_snapshot.is_some() {
        snapshot
            .and_then(|snapshot| snapshot.updated_by_user_id)
            .map(|user_id| user_id.to_string())
            .unwrap_or_default()
    } else {
        page.updated_by
            .map(|user_id| user_id.to_string())
            .unwrap_or_default()
    };
    let commented_by = runtime
        .and_then(|runtime| runtime.commented_by_name.clone())
        .or_else(|| snapshot.and_then(|snapshot| snapshot.commented_by_name.clone()))
        .unwrap_or_default();
    let commented_by_slug = runtime.and_then(|runtime| runtime.commented_by_slug.clone());
    let commented_by_id = runtime
        .and_then(|runtime| runtime.commented_by_user_id)
        .map(|user_id| user_id.to_string());
    let commented_by_linked = runtime
        .and_then(|runtime| {
            runtime.commented_by_user_id.map(|user_id| {
                let display = WikidotUserDisplay {
                    user_id,
                    name: runtime.commented_by_name.clone().unwrap_or_default(),
                    slug: runtime.commented_by_slug.clone(),
                    wikidot_profile: runtime.commented_by_wikidot_profile,
                };
                render_list_pages_wikidot_user(user_id, Some(&display))
            })
        })
        .or_else(|| {
            runtime
                .and_then(|runtime| runtime.commented_by_name.as_deref())
                .map(render_list_pages_snapshot_user)
        })
        .or_else(|| {
            snapshot
                .and_then(|snapshot| snapshot.commented_by_name.as_deref())
                .map(render_list_pages_snapshot_user)
        })
        .unwrap_or_default();
    let created_at = snapshot
        .map(|snapshot| snapshot.created_at)
        .or(page.created_at);
    let updated_at = snapshot
        .map(|snapshot| snapshot.updated_at)
        .or(page.updated_at);
    let commented_at = runtime
        .and_then(|runtime| runtime.commented_at)
        .or_else(|| snapshot.and_then(|snapshot| snapshot.commented_at));
    let comments = runtime
        .map(|runtime| runtime.comments.to_string())
        .or_else(|| snapshot.map(|snapshot| snapshot.comments.to_string()))
        .unwrap_or_else(|| "0".to_owned());
    let tags = page.tags.as_deref().unwrap_or(&[]);
    let visible_tags = tags
        .iter()
        .filter(|tag| is_list_pages_visible_tag(tag))
        .cloned()
        .collect::<Vec<_>>();
    let hidden_tags = tags
        .iter()
        .filter(|tag| is_list_pages_hidden_tag(tag))
        .cloned()
        .collect::<Vec<_>>();
    let tags_text = visible_tags.join(" ");
    let rating = if runtime.is_some_and(|runtime| runtime.rating_type == "stars") {
        let rating = format_list_pages_rating(page.score);
        protect_list_pages_generated_html(
            format!(
                "<span class=\"page-rate-list-pages-start\" data-rating=\"{rating}\" data-wikijump-compat-listpages-rating=\"1\">{rating}</span>",
            ),
            context.render_generated_html,
            compat_html,
        )
    } else {
        format_list_pages_rating(page.score)
    };
    let rating_percent = if runtime.is_some_and(|runtime| runtime.rating_type == "stars")
    {
        format_list_pages_rating(page.score.map(|score| score * 20.0).or(Some(0.0)))
    } else {
        String::new()
    };
    // The frozen corpus predates vote-count capture. Keep this value typed as
    // optional provenance and select the component's explicit zero-vote state
    // when it is absent; inventing a count from the net rating would create a
    // visibly plausible but false upvote/downvote ratio.
    let rating_votes = runtime
        .map(|runtime| runtime.rating_votes)
        .or_else(|| snapshot.and_then(|snapshot| snapshot.rating_votes))
        .unwrap_or(0)
        .to_string();
    let index = index.to_string();
    let total_or_limit = context
        .authored_limit
        .map_or(total, |limit| total.min(limit as usize))
        .to_string();
    let total = total.to_string();
    let authored_limit = context
        .authored_limit
        .map(|limit| limit.to_string())
        .unwrap_or_default();
    let summary_html = if context.default_summary_first_paragraph {
        context
            .page_rendered_first_paragraph
            .or(context.page_rendered_summary)
    } else {
        context
            .page_rendered_summary
            .or(context.page_rendered_first_paragraph)
    };
    let summary = summary_html.or(context.page_rendered_content).map_or_else(
        || {
            let section = context
                .page_wikitext
                .map(|wikitext| wikidot_content_section(wikitext, Some(1)))
                .unwrap_or_default();
            if context.default_summary_first_paragraph {
                list_pages_first_paragraph(&section).to_owned()
            } else {
                section
            }
        },
        |html| {
            push_list_pages_rendered_fragment_with_mode(
                html,
                compat_html,
                context.page_rendered_summary_is_block,
            )
        },
    );
    let first_paragraph = context.page_rendered_first_paragraph.map_or_else(
        || {
            context
                .page_wikitext
                .map(|wikitext| {
                    let section = wikidot_content_section(wikitext, Some(1));
                    list_pages_first_paragraph(&section).to_owned()
                })
                .unwrap_or_else(|| list_pages_first_paragraph(&summary).to_owned())
        },
        |html| push_list_pages_rendered_fragment(html, compat_html),
    );
    let mut source_cursor = 0usize;
    let mut substituted_cursor = 0usize;
    let substituted = LISTPAGES_VARIABLE_REGEX
        .replace_all(template, |captures: &regex::Captures<'_>| {
            let matched = captures
                .get(0)
                .expect("ListPages variable capture exists");
            let inside_html_block = html_body_ranges
                .iter()
                .any(|range| range.start <= matched.start() && matched.end() <= range.end);
            substituted_cursor += matched.start() - source_cursor;
            let replacement_start = substituted_cursor;
            let runtime_origin = if generated_slots.is_some()
                && runtime_text_ranges.is_some()
            {
                if list_pages_variable_capture_has_unknown_name(captures) {
                    Some(ftml::delayed::TextOrigin::RuntimeLiteral)
                } else if list_pages_variable_capture_is_valid(captures)
                    && captures["name"].eq_ignore_ascii_case("title")
                    && runtime_title_is_delayed
                {
                    Some(ftml::delayed::TextOrigin::RuntimeScalar)
                } else {
                    None
                }
            } else {
                None
            };
            let mut replacement = if !list_pages_variable_capture_is_valid(captures) {
                captures[0].to_owned()
            } else {
                match captures["name"].to_ascii_lowercase().as_str() {
                "title_linked" | "linked_title"
                    if let Some(slots) = generated_slots.as_mut() =>
                {
                    let marker = captures[0].to_owned();
                    slots.push(ListPagesGeneratedSlot {
                        source_range: replacement_start
                            ..replacement_start + marker.len(),
                        value: ftml::delayed::GeneratedValue::PageLink {
                            page: ftml::data::PageRef::page_only(full_slug),
                            label: Cow::Owned(title.clone()),
                        },
                    });
                    marker
                }
                "title_linked" | "linked_title" => title_linked.clone(),
                "title" => title_plain.clone(),
                "name" | "slug" | "page_name" => slug.to_owned(),
                "fullname" | "full_slug" | "page_unix_name" | "full_page_name"
                    if list_pages_variable_starts_triple_link_target(
                        template,
                        captures
                            .get(0)
                            .expect("ListPages variable capture exists")
                            .start(),
                    ) =>
                {
                    format!("/{full_slug}")
                }
                "fullname" | "full_slug" | "page_unix_name" | "full_page_name" => {
                    full_slug.to_owned()
                }
                "link" if !slug.is_empty() && !context.site.is_empty() => link.clone(),
                "link" => captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned(),
                "created_by" | "createdby" => created_by.clone(),
                "created_by_linked" | "createdbylinked" | "author" => {
                    protect_list_pages_generated_html(
                        created_by_linked.clone(),
                        context.render_generated_html,
                        compat_html,
                    )
                }
                "created_by_unix" => created_by_unix
                    .clone()
                    .unwrap_or_else(|| captures[0].to_owned()),
                "created_by_id" => created_by_id.clone(),
                "created_at" | "createdat" | "date"
                    if inside_html_block && captures.name("format").is_none() =>
                {
                    created_at.map_or_else(String::new, |created_at| {
                        format!("%%date|{}%%", created_at.unix_timestamp())
                    })
                }
                "created_at" | "createdat" | "date" => {
                    protect_list_pages_generated_html(
                        format_list_pages_created_at(
                            created_at,
                            captures.name("format").map(|matched| matched.as_str()),
                            context.render_generated_html,
                            context.page_preview,
                        ),
                        context.render_generated_html && !context.page_preview,
                        compat_html,
                    )
                }
                "updated_by" | "updatedby" => updated_by.clone(),
                "updated_by_linked"
                | "updatedbylinked"
                | "author_edited"
                | "user_edited" => {
                    protect_list_pages_generated_html(
                        updated_by_linked.clone(),
                        context.render_generated_html,
                        compat_html,
                    )
                }
                "updated_by_unix" => updated_by_slug.clone(),
                "updated_by_id" => updated_by_id.clone(),
                "updated_at" | "updatedat" | "date_edited" => {
                    protect_list_pages_generated_html(
                        format_list_pages_created_at(
                            updated_at,
                            captures.name("format").map(|matched| matched.as_str()),
                            context.render_generated_html,
                            context.page_preview,
                        ),
                        context.render_generated_html && !context.page_preview,
                        compat_html,
                    )
                }
                "commented_by"
                | "commentedby" => commented_by.clone(),
                "commented_by_linked" | "commentedbylinked" => {
                    protect_list_pages_generated_html(
                        commented_by_linked.clone(),
                        context.render_generated_html,
                        compat_html,
                    )
                }
                "commented_by_unix" | "commented_by_id" if commented_by.is_empty() => {
                    String::new()
                }
                "commented_by_unix" => commented_by_slug
                    .clone()
                    .unwrap_or_else(|| captures[0].to_owned()),
                "commented_by_id" => commented_by_id
                    .clone()
                    .unwrap_or_else(|| captures[0].to_owned()),
                "commented_at" | "commentedat" => protect_list_pages_generated_html(
                    format_list_pages_created_at(
                        commented_at,
                        captures.name("format").map(|matched| matched.as_str()),
                        context.render_generated_html,
                        context.page_preview,
                    ),
                    context.render_generated_html && !context.page_preview,
                    compat_html,
                ),
                "rating" => rating.clone(),
                "rating_votes" | "ratingvotes" => rating_votes.clone(),
                "comments" => comments.clone(),
                "tags" => tags_text.clone(),
                "tags_linked" | "tagslinked"
                    if generated_slots.is_some()
                        && captures.name("format").is_none()
                        && context.tag_target.is_none() =>
                {
                    let marker = captures[0].to_owned();
                    generated_slots
                        .as_mut()
                        .expect("generated slot registry exists")
                        .push(ListPagesGeneratedSlot {
                            source_range: replacement_start
                                ..replacement_start + marker.len(),
                            value: ftml::delayed::GeneratedValue::TagLinks {
                                tags: Cow::Owned(
                                    visible_tags
                                        .iter()
                                        .map(|tag| ftml::delayed::ResolvedTagRef {
                                            tag: Cow::Owned(tag.clone()),
                                        })
                                        .collect(),
                                ),
                                separator: Cow::Borrowed(" "),
                            },
                        });
                    marker
                }
                "tags_linked" | "tagslinked" => {
                    let tag_target_prefix;
                    let path_prefix = match captures.name("format") {
                        Some(format) => Some(format.as_str()),
                        None => {
                            tag_target_prefix = context
                                .tag_target
                                .and_then(list_pages_tag_target_prefix);
                            tag_target_prefix.as_deref()
                        }
                    };
                    render_list_pages_tags(
                        &visible_tags,
                        path_prefix,
                        context.render_generated_html,
                        compat_html,
                    )
                }
                "_tags_linked" => render_list_pages_tags(
                    &hidden_tags,
                    captures.name("format").map(|matched| matched.as_str()),
                    context.render_generated_html,
                    compat_html,
                ),
                "_tags" => hidden_tags.join(" "),
                "category" => context.category.to_owned(),
                "size" => context
                    .page_wikitext_scalar_count
                    .map(|scalar_count| scalar_count.to_string())
                    .unwrap_or_else(|| captures[0].to_owned()),
                "children" => context
                    .page_child_count
                    .map(|child_count| child_count.to_string())
                    .unwrap_or_else(|| captures[0].to_owned()),
                "revisions" => context
                    .page_revision_count
                    .map(|revision_count| revision_count.to_string())
                    .unwrap_or_else(|| captures[0].to_owned()),
                "site_domain" if !context.site.is_empty() => {
                    format!("{}.wikidot.com", context.site)
                }
                "site_domain" => captures[0].to_owned(),
                "site_title" => context.site_title.to_owned(),
                "site_name" => context.site.to_owned(),
                "parent_fullname" => {
                    context.page_parent_fullname.unwrap_or("").to_owned()
                }
                "parent_name" => context
                    .page_parent_display
                    .map(|parent| parent.name.clone())
                    .or_else(|| {
                        context.page_parent_fullname.map(|fullname| {
                            fullname
                                .split_once(':')
                                .map_or(fullname, |(_, name)| name)
                                .to_owned()
                        })
                    })
                    .unwrap_or_default(),
                "parent_category" => context
                    .page_parent_display
                    .map(|parent| parent.category.clone())
                    .or_else(|| {
                        context
                            .page_parent_fullname
                            .and_then(|fullname| fullname.split_once(':'))
                            .map(|(category, _)| category.to_owned())
                    })
                    .unwrap_or_default(),
                "parent_title" => context
                    .page_parent_display
                    .map(|parent| sanitize_list_pages_title(&parent.title))
                    .unwrap_or_default(),
                "parent_title_linked" => context
                    .page_parent_display
                    .map(|parent| {
                        let parent_title = sanitize_list_pages_title(&parent.title);
                        render_list_pages_linked_title(
                            &parent.fullname,
                            &parent_title,
                            compat_text,
                        )
                    })
                    .unwrap_or_default(),
                "rating_percent"
                    if runtime
                        .is_some_and(|runtime| runtime.rating_type == "stars") =>
                {
                    rating_percent.clone()
                }
                // Live Wikidot leaves this variable unsubstituted on a
                // plus/minus site, so the authored text survives rather than
                // collapsing to an empty cell.
                "rating_percent" => captures[0].to_owned(),
                "form_data" => captures
                    .name("argument")
                    .map(|matched| matched.as_str())
                    .and_then(|field| {
                        substitute_list_pages_form_data(
                            field,
                            context.data_form_values,
                            context.data_form_definition,
                        )
                    })
                    .map(|value| {
                        if context
                            .data_form_definition
                            .and_then(|definition| definition.field(
                                captures["argument"].as_ref(),
                            ))
                            .is_some_and(|field| field.field_type.as_deref() == Some("wiki"))
                        {
                            render_list_pages_form_wiki_value(
                                &value,
                                context.fallback_link_titles,
                                compat_html,
                            )
                        } else {
                            value
                        }
                    })
                    .unwrap_or_else(|| captures[0].to_owned()),
                "form_raw" => captures
                    .name("argument")
                    .map(|matched| matched.as_str())
                    .and_then(|field| {
                        substitute_list_pages_form_raw(
                            field,
                            context.data_form_values,
                            context.data_form_definition,
                        )
                    })
                    .map(|value| {
                        if context
                            .data_form_definition
                            .and_then(|definition| definition.field(
                                captures["argument"].as_ref(),
                            ))
                            .is_some_and(|field| field.field_type.as_deref() == Some("wiki"))
                        {
                            render_list_pages_form_wiki_value(
                                &value,
                                context.fallback_link_titles,
                                compat_html,
                            )
                        } else {
                            value
                        }
                    })
                    .unwrap_or_else(|| captures[0].to_owned()),
                "form_label" => captures
                    .name("argument")
                    .map(|matched| matched.as_str())
                    .and_then(|field| {
                        substitute_list_pages_form_label(
                            field,
                            context.data_form_definition,
                        )
                    })
                    .unwrap_or_else(|| captures[0].to_owned()),
                "form_hint" => captures
                    .name("argument")
                    .map(|matched| matched.as_str())
                    .and_then(|field| {
                        substitute_list_pages_form_hint(
                            field,
                            context.data_form_definition,
                        )
                    })
                    .unwrap_or_else(|| captures[0].to_owned()),
                "content" | "text" | "long" | "body" => {
                    let section = captures
                        .name("argument")
                        .and_then(|matched| matched.as_str().parse().ok());
                    if section.is_none()
                        && let Some(rendered_content) =
                            context.page_rendered_content
                    {
                        push_list_pages_rendered_fragment(rendered_content, compat_html)
                    } else {
                        context.page_wikitext.map(|wikitext| {
                            protect_list_pages_content_insertion(
                                &wikidot_content_section(wikitext, section),
                                compat_text,
                                tracked_content_fragments.as_deref_mut(),
                            )
                        })
                        .unwrap_or_default()
                    }
                }
                "summary" | "description" | "short" => summary.to_owned(),
                "first_paragraph" => first_paragraph.clone(),
                "preview" => {
                    let preview = context
                        .page_compiled_body_html
                        .map(|compiled_html| {
                            list_pages_preview(
                                &list_pages_plain_text(compiled_html),
                                captures
                                    .name("length")
                                    .and_then(|length| {
                                        list_pages_preview_length(length.as_str())
                                    }),
                            )
                        })
                        .unwrap_or_default();
                    protect_list_pages_generated_html(
                        format!(
                            r#"<span data-wikijump-compat-listpages-preview="1" style="white-space: pre-wrap;">{}</span>"#,
                            escape_html_text(&preview),
                        ),
                        context.render_generated_html,
                        compat_html,
                    )
                }
                "index" => index.clone(),
                "total" => total.clone(),
                "limit" => authored_limit.clone(),
                "total_or_limit" => total_or_limit.clone(),
                _ => captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned(),
                }
            };
            if generated_slots.is_some() {
                neutralize_authored_markers(&mut replacement);
            }
            if let Some(origin) = runtime_origin
                && !replacement.is_empty()
                && let Some(ranges) = runtime_text_ranges.as_mut()
            {
                ranges.push(ListPagesRuntimeTextRange {
                    source_range: replacement_start
                        ..replacement_start + replacement.len(),
                    origin,
                });
            }
            source_cursor = matched.end();
            substituted_cursor = replacement_start + replacement.len();
            replacement
        })
        .into_owned();

    if generated_slots.is_some() {
        substituted
    } else {
        RenderService::resolve_wikidot_parser_functions(&substituted)
    }
}

#[cfg(test)]
mod tests {
    use crate::services::render::compat::CompatHtmlFragments;

    use super::{
        list_pages_rendered_fragment_has_html_block, list_pages_rendered_inline_fragment,
        push_list_pages_rendered_fragment, push_list_pages_rendered_fragment_with_mode,
    };

    #[test]
    fn rendered_inline_fragment_keeps_empty_paragraph_boundaries() {
        let source = "<p>before</p>\n<p>\n\n</p>\n<p>after</p>";

        assert_eq!(
            list_pages_rendered_inline_fragment(source),
            "before</p>\n\n\n\n<p>after",
        );
    }

    #[test]
    fn rendered_inline_fragment_still_flattens_ordinary_paragraphs() {
        let source = "<p>before</p>\n<p>after</p>";

        assert_eq!(list_pages_rendered_inline_fragment(source), "before\nafter");
    }

    #[test]
    fn rendered_block_fragment_drops_one_redundant_outer_paragraph() {
        let mut compat_html = CompatHtmlFragments::new("");
        let marker = push_list_pages_rendered_fragment(
            r#"<p><p><iframe src="/page/html/hash-1" class="html-block-iframe"></iframe></p></p>"#,
            &mut compat_html,
        );

        let restored = compat_html.restore(&format!("<div><p>{marker}</p></div>"));

        assert_eq!(
            restored,
            r#"<div><p><iframe src="/page/html/hash-1" class="html-block-iframe"></iframe></p></div>"#,
        );
    }

    #[test]
    fn rendered_style_frame_fragment_escapes_the_outer_row_paragraph() {
        let mut compat_html = CompatHtmlFragments::new("");
        let marker = push_list_pages_rendered_fragment(
            r#"<p><p><iframe src="/-/wikidot-interwiki/styleFrame.html?priority=1"></iframe></p><p>body</p></p>"#,
            &mut compat_html,
        );

        let restored = compat_html.restore(&format!("<div><p>{marker}</p></div>"));

        assert_eq!(
            restored,
            r#"<div><p><iframe src="/-/wikidot-interwiki/styleFrame.html?priority=1"></iframe></p><p>body</p></div>"#,
        );
    }

    #[test]
    fn empty_paragraph_restoration_does_not_rescan_the_output_per_marker() {
        let mut source = String::from("<p>before</p>\n");
        for _ in 0..1_024 {
            source.push_str("<p>\n</p>\n");
        }
        source.push_str("<p>after</p>");

        let rendered = list_pages_rendered_inline_fragment(&source);
        let scanned_bytes = super::take_empty_paragraph_restore_scanned_bytes();

        assert!(rendered.contains("before"), "{rendered:?}");
        assert!(rendered.contains("after"), "{rendered:?}");
        assert!(
            scanned_bytes <= source.len() * 4,
            "empty paragraph restoration rescanned {scanned_bytes} bytes for {} source bytes",
            source.len(),
        );
    }

    #[test]
    fn rendered_fragment_recognizes_only_the_trusted_html_block_iframe() {
        assert!(list_pages_rendered_fragment_has_html_block(
            r#"<p>HTML_START</p><p><iframe src="/page/html/hash-1" class="html-block-iframe"></iframe></p>"#,
        ));
        assert!(!list_pages_rendered_fragment_has_html_block(
            r#"<p><iframe src="/user-content/frame"></iframe></p>"#,
        ));
        assert!(!list_pages_rendered_fragment_has_html_block(
            r#"<p><iframe src="/page/html/hash-1" class="other-iframe"></iframe></p>"#,
        ));
        assert!(list_pages_rendered_fragment_has_html_block(
            r#"<p><iframe src="/-/wikidot-interwiki/styleFrame.html?priority=1"></iframe></p>"#,
        ));
    }

    #[test]
    fn forced_block_rendered_fragment_escapes_the_outer_row_paragraph() {
        let mut compat_html = CompatHtmlFragments::new("");
        let marker = push_list_pages_rendered_fragment_with_mode(
            r#"<div style="text-align: right;"><div class="page-rate-widget-box"></div></div><p>body</p>"#,
            &mut compat_html,
            true,
        );

        let restored = compat_html.restore(&format!("<p>{marker}</p>"));
        assert_eq!(
            restored,
            r#"<div style="text-align: right;"><div class="page-rate-widget-box"></div></div><p>body</p>"#,
        );
    }

    #[test]
    fn rendered_html_block_fragment_preserves_wikidot_paragraph_boundaries() {
        let mut compat_html = CompatHtmlFragments::new("");
        let marker = push_list_pages_rendered_fragment(
            concat!(
                "<p>DOC_HTML_BEGIN</p>\n",
                "<p><iframe src=\"/page/html/hash-1\" ",
                "class=\"html-block-iframe\"></iframe></p>",
            ),
            &mut compat_html,
        );

        let restored = compat_html.restore(&format!("<div><p>{marker}</p></div>"));

        assert_eq!(
            restored,
            concat!(
                "<div><p>DOC_HTML_BEGIN</p>\n",
                "<p><iframe src=\"/page/html/hash-1\" ",
                "class=\"html-block-iframe\"></iframe></p></div>",
            ),
        );
    }
}
