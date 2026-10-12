//! Wikidot RatedPages runtime expansion.

use super::{
    AuthorSelector, BTreeMap, CategoriesSelector, ComparisonOperation,
    CompatHtmlFragments, Cow, DateSelector, FoundPageFields, IncludedCategories,
    ListPagesRuntimeDisplay, LiteralRegionIndex, MAX_LISTPAGES_RENDER_SCAN_ROWS,
    OrderBySelector, OrderProperty, PageParentSelector, PageQuery, PageTypeSelector,
    PaginationSelector, RATEDPAGES_MODULE_REGEX, RangeSelector, RenderService, Result,
    ScoreSelector, ScoreValue, ServiceContext, TagCondition, WikitextSettings,
    escape_list_pages_html_attr, escape_list_pages_html_text,
    find_viewable_list_pages_rows_with_batch_floor,
    wikidot_module_arguments_ignoring_bare_flags,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RatedPagesOrder {
    RatingDesc,
    RatingAsc,
    DateCreatedDesc,
    DateCreatedAsc,
}

#[derive(Clone, Debug)]
struct RatedPagesArguments {
    category: Option<String>,
    order: RatedPagesOrder,
    min_rating: Option<i64>,
    max_rating: Option<i64>,
    limit: usize,
    comments: bool,
}

fn parse_rated_pages_arguments(head: &str) -> Option<RatedPagesArguments> {
    let parsed = wikidot_module_arguments_ignoring_bare_flags(head)?;
    let mut arguments = RatedPagesArguments {
        category: None,
        order: RatedPagesOrder::RatingDesc,
        min_rating: None,
        max_rating: None,
        limit: 10,
        comments: false,
    };

    for argument in parsed {
        let value = argument.value.trim();
        match argument.key.to_ascii_lowercase().as_str() {
            "category" => {
                arguments.category = (!value.is_empty()).then(|| value.to_owned());
            }
            "order" => {
                arguments.order = match value.to_ascii_lowercase().as_str() {
                    "rating-asc" | "rate-asc" => RatedPagesOrder::RatingAsc,
                    "rating-desc" | "rate-desc" => RatedPagesOrder::RatingDesc,
                    "date-created-asc" => RatedPagesOrder::DateCreatedAsc,
                    "date-created-desc" => RatedPagesOrder::DateCreatedDesc,
                    _ => RatedPagesOrder::RatingDesc,
                };
            }
            "minrating" => {
                arguments.min_rating = value.parse().ok();
            }
            "maxrating" => {
                arguments.max_rating = value.parse().ok();
            }
            "limit" => {
                if let Ok(limit) = value.parse::<usize>()
                    && limit > 0
                {
                    arguments.limit = limit;
                }
            }
            "comments" if argument.key == "comments" => {
                arguments.comments = !value.is_empty();
            }
            _ => {}
        }
    }

    Some(arguments)
}

fn rated_pages_score_selectors(arguments: &RatedPagesArguments) -> Vec<ScoreSelector> {
    let mut selectors = Vec::with_capacity(2);
    if let Some(min_rating) = arguments.min_rating {
        selectors.push(ScoreSelector {
            score: ScoreValue::Integer(min_rating),
            comparison: ComparisonOperation::GreaterOrEqualThan,
        });
    }
    if let Some(max_rating) = arguments.max_rating {
        selectors.push(ScoreSelector {
            score: ScoreValue::Integer(max_rating),
            comparison: ComparisonOperation::LessOrEqualThan,
        });
    }
    selectors
}

fn rated_pages_order(order: RatedPagesOrder) -> OrderBySelector {
    match order {
        RatedPagesOrder::RatingDesc => OrderBySelector {
            property: OrderProperty::Score,
            ascending: false,
        },
        RatedPagesOrder::RatingAsc => OrderBySelector {
            property: OrderProperty::Score,
            ascending: true,
        },
        RatedPagesOrder::DateCreatedDesc => OrderBySelector {
            property: OrderProperty::CreatedAt,
            ascending: false,
        },
        RatedPagesOrder::DateCreatedAsc => OrderBySelector {
            property: OrderProperty::CreatedAt,
            ascending: true,
        },
    }
}

fn format_rated_pages_score(score: f32) -> String {
    if score.fract() == 0.0 {
        (score as i64).to_string()
    } else {
        score.to_string()
    }
}

fn render_rated_pages_module(
    rows: &[crate::services::page_query::FoundPageRow],
    include_comments: bool,
    runtime_displays: &BTreeMap<i64, ListPagesRuntimeDisplay>,
) -> String {
    let mut output = String::from(
        "<div class=\"top-rated-pages-box\">\n\n\t<div class=\"top-rated-pages-list\">\n",
    );
    for row in rows {
        let slug = row.slug.as_deref().unwrap_or_default();
        let title = row.title.as_deref().unwrap_or(slug);
        let rating = format_rated_pages_score(row.score.unwrap_or(0.0));
        let comment_count = runtime_displays
            .get(&row.page_id)
            .map_or(0, |display| display.comments);
        output.push_str("\t\t\t\t\t<div class=\"list-item\">\n");
        output.push_str(&format!(
            "\t\t\t\t<a href=\"/{}\">{}</a>\n",
            escape_list_pages_html_attr(slug),
            escape_list_pages_html_text(title),
        ));
        let label = if include_comments {
            format!("Rating: {rating}, Comments: {comment_count}")
        } else {
            format!("Rating: {rating}")
        };
        output.push_str(&format!(
            "\t\t\t\t<span style=\"color: #777\">({})</span>\n",
            escape_list_pages_html_text(&label),
        ));
        output.push_str("\t\t\t</div>\n");
    }
    output.push_str("\t\t\t</div>\n\n</div>");
    output
}

impl RenderService {
    pub(super) async fn expand_rated_pages_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        viewer_user_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !RATEDPAGES_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }
        let Some(current_site_id) = current_site_id else {
            return Ok(wikitext);
        };

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in RATEDPAGES_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a RatedPages capture always has a complete match");
            let module = captures
                .name("module")
                .expect("a RatedPages capture always has a module invocation");
            if literal_regions.contains(module.start()) {
                continue;
            }
            let head = captures.name("head").map_or("", |head| head.as_str());
            let Some(arguments) = parse_rated_pages_arguments(head) else {
                continue;
            };
            expanded.push_str(&wikitext[cursor..matched.start()]);
            let rendered = Self::render_rated_pages_query(
                ctx,
                current_site_id,
                viewer_user_id,
                &arguments,
            )
            .await?;
            expanded.push_str(&compat_html.push_block_html(rendered));
            cursor = matched.end();
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn render_rated_pages_query(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        viewer_user_id: Option<i64>,
        arguments: &RatedPagesArguments,
    ) -> Result<String> {
        let categories = arguments
            .category
            .as_deref()
            .map(|category| vec![Cow::Borrowed(category)]);
        let included_categories = categories
            .as_deref()
            .map_or(IncludedCategories::All, IncludedCategories::List);
        let score = rated_pages_score_selectors(arguments);
        let score_order = matches!(
            arguments.order,
            RatedPagesOrder::RatingAsc | RatedPagesOrder::RatingDesc
        );
        let query_limit = if score_order {
            u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS)
        } else {
            arguments.limit as u64
        };
        let query = PageQuery {
            current_page_id: 0,
            current_site_id,
            queried_site_id: None,
            page_type: PageTypeSelector::Normal,
            categories: CategoriesSelector {
                included_categories,
                excluded_categories: &[],
            },
            tags: TagCondition {
                any_present: &[],
                all_present: &[],
                none_present: &[],
                untagged: false,
            },
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            author: AuthorSelector::All,
            score: &score,
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs: &[],
            data_form_fields: &[],
            order: Some(rated_pages_order(arguments.order)),
            candidate_limit: Some(query_limit),
            pagination: PaginationSelector {
                limit: Some(query_limit),
                per_page: PaginationSelector::default().per_page,
                reversed: false,
            },
            variables: &[],
            fields: FoundPageFields {
                title: true,
                slug: true,
                page_category_id: true,
                score: true,
                ..FoundPageFields::default()
            },
        };
        let mut permission_cache = BTreeMap::new();
        let rows = find_viewable_list_pages_rows_with_batch_floor(
            ctx,
            viewer_user_id,
            query,
            arguments.limit,
            &mut permission_cache,
            None,
            super::super::service::MAX_LISTPAGES_RENDER_LIMIT,
        )
        .await?;
        let runtime_displays = if arguments.comments {
            Self::load_list_pages_runtime_displays(ctx, &rows.pages.pages).await?
        } else {
            BTreeMap::new()
        };
        Ok(render_rated_pages_module(
            &rows.pages.pages,
            arguments.comments,
            &runtime_displays,
        ))
    }
}
