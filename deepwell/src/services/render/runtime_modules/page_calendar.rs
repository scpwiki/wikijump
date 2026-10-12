//! Wikidot PageCalendar runtime expansion.

use super::{
    Action, BTreeMap, BTreeSet, CheckPermissionContext, CompatHtmlFragments,
    ConnectionTrait, Cow, Error, ErrorType, FromQueryResult, Future, HashMap,
    LiteralRegionIndex, PAGE_CALENDAR_CATEGORY_ERROR, PAGECALENDAR_MODULE_REGEX,
    PageInfo, Permission, PermissionService, Reference, RenderService, Resource, Result,
    ResultExt, ServiceContext, Statement, UrlArguments, Value, WikitextSettings,
    escape_list_pages_html_attr, escape_list_pages_html_text, wikidot_module_arguments,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PageCalendarCategorySelector {
    All,
    Names(Vec<String>),
}

type PageCalendarCounts = Option<BTreeMap<i32, BTreeMap<u8, usize>>>;

#[derive(Default)]
struct PageCalendarCountsCache {
    counts: HashMap<PageCalendarCategorySelector, PageCalendarCounts>,
}

impl PageCalendarCountsCache {
    async fn get_or_init<F, Fut>(
        &mut self,
        category: PageCalendarCategorySelector,
        load: F,
    ) -> Result<PageCalendarCounts>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<PageCalendarCounts>>,
    {
        if let Some(counts) = self.counts.get(&category) {
            return Ok(counts.clone());
        }

        let counts = load().await?;
        self.counts.insert(category, counts.clone());
        Ok(counts)
    }
}

#[derive(Clone, Debug)]
struct PageCalendarArguments {
    categories: PageCalendarCategorySelector,
    category_url_value: Option<String>,
    tags_url_value: Option<String>,
    selected_date: Option<String>,
    target_page: String,
    url_attr_prefix: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PageCalendarExpansionOptions<'a> {
    pub(super) current_site_id: Option<i64>,
    pub(super) current_page_id: Option<i64>,
    pub(super) url: UrlArguments<'a>,
}

#[derive(Debug, FromQueryResult)]
struct PageCalendarCategoryRow {
    category_id: i64,
    slug: String,
}

#[derive(Debug, FromQueryResult)]
struct PageCalendarPageRow {
    page_id: i64,
    page_category_id: i64,
    created_at: time::OffsetDateTime,
    tags: Vec<String>,
}

fn current_page_calendar_category(page_info: &PageInfo<'_>) -> String {
    page_info
        .category
        .as_deref()
        .unwrap_or("_default")
        .to_owned()
}

fn current_page_calendar_target(page_info: &PageInfo<'_>) -> String {
    if page_info.page.contains(':') {
        page_info.page.to_string()
    } else if let Some(category) = page_info
        .category
        .as_deref()
        .filter(|category| *category != "_default")
    {
        format!("{category}:{}", page_info.page)
    } else {
        page_info.page.to_string()
    }
}

fn parse_page_calendar_arguments(
    head: &str,
    page_info: &PageInfo<'_>,
    url: UrlArguments<'_>,
) -> Option<PageCalendarArguments> {
    let parsed = wikidot_module_arguments(head)?;
    let url_attr_prefix = parsed
        .iter()
        .filter(|argument| argument.key.eq_ignore_ascii_case("urlattrprefix"))
        .map(|argument| argument.value.trim())
        .rfind(|prefix| !prefix.is_empty())
        .map(str::to_owned);
    let mut target_page = current_page_calendar_target(page_info);
    let mut category_value = None::<&str>;
    let mut tags_value = None::<&str>;

    for argument in &parsed {
        let value = argument.value.trim();
        match argument.key.to_ascii_lowercase().as_str() {
            "targetpage" | "startpage" if !value.is_empty() => {
                target_page = value.to_owned();
            }
            "category" => category_value = Some(value),
            "tags" => tags_value = Some(value),
            _ => {}
        }
    }

    let (category_value, category_from_url) = match category_value {
        Some(value) => match resolve_page_calendar_url_selector(
            value,
            url.value_for_list_pages_argument(url_attr_prefix.as_deref(), "category"),
        ) {
            PageCalendarUrlSelector::Value { value, from_url } => (value, from_url),
            PageCalendarUrlSelector::Dropped => {
                (current_page_calendar_category(page_info), false)
            }
        },
        None => (current_page_calendar_category(page_info), false),
    };
    let categories = parse_page_calendar_categories(&category_value);
    let category_url_value = category_from_url.then_some(category_value);

    let tags_url_value = tags_value.and_then(|value| {
        match resolve_page_calendar_url_selector(
            value,
            url.value_for_list_pages_argument(url_attr_prefix.as_deref(), "tags"),
        ) {
            PageCalendarUrlSelector::Value { value, .. } => {
                (!value.trim().is_empty()).then(|| page_calendar_link_tag_value(&value))
            }
            PageCalendarUrlSelector::Dropped => None,
        }
    });
    let selected_date = url
        .value_for_list_pages_argument(url_attr_prefix.as_deref(), "date")
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_owned());

    Some(PageCalendarArguments {
        categories,
        category_url_value,
        tags_url_value,
        selected_date,
        target_page,
        url_attr_prefix,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PageCalendarUrlSelector {
    Value { value: String, from_url: bool },
    Dropped,
}

fn resolve_page_calendar_url_selector(
    value: &str,
    url_value: Option<&str>,
) -> PageCalendarUrlSelector {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("@URL") {
        return url_value.map_or(PageCalendarUrlSelector::Dropped, |value| {
            PageCalendarUrlSelector::Value {
                value: value.to_owned(),
                from_url: true,
            }
        });
    }

    if trimmed
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("@URL|"))
    {
        return url_value.map_or_else(
            || PageCalendarUrlSelector::Value {
                value: trimmed[5..].to_owned(),
                from_url: false,
            },
            |value| PageCalendarUrlSelector::Value {
                value: value.to_owned(),
                from_url: true,
            },
        );
    }

    PageCalendarUrlSelector::Value {
        value: trimmed.to_owned(),
        from_url: false,
    }
}

fn parse_page_calendar_categories(value: &str) -> PageCalendarCategorySelector {
    let names = split_page_calendar_values(value)
        .into_iter()
        .filter(|category| !category.is_empty())
        .collect::<Vec<_>>();
    if names.iter().any(|category| category == "*") {
        PageCalendarCategorySelector::All
    } else {
        PageCalendarCategorySelector::Names(names)
    }
}

fn split_page_calendar_values(value: &str) -> Vec<String> {
    value
        .split(|character: char| character.is_whitespace() || character == ',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn page_calendar_link_tag_value(value: &str) -> String {
    value.trim().replace('+', " ")
}

fn page_calendar_argument_key(prefix: Option<&str>, name: &str) -> String {
    match prefix.map(str::trim).filter(|prefix| !prefix.is_empty()) {
        Some(prefix) => format!("{prefix}_{name}"),
        None => name.to_owned(),
    }
}

fn page_calendar_path(arguments: &PageCalendarArguments, date: &str) -> String {
    let mut path = String::from("/");
    path.push_str(&escape_list_pages_html_attr(&arguments.target_page));
    if let Some(tags) = &arguments.tags_url_value {
        path.push('/');
        path.push_str(&escape_list_pages_html_attr(&page_calendar_argument_key(
            arguments.url_attr_prefix.as_deref(),
            "tag",
        )));
        path.push('/');
        path.push_str(&escape_list_pages_html_attr(tags));
    }
    if let Some(category) = &arguments.category_url_value {
        path.push('/');
        path.push_str(&escape_list_pages_html_attr(&page_calendar_argument_key(
            arguments.url_attr_prefix.as_deref(),
            "category",
        )));
        path.push('/');
        path.push_str(&escape_list_pages_html_attr(category));
    }
    path.push('/');
    path.push_str(&escape_list_pages_html_attr(&page_calendar_argument_key(
        arguments.url_attr_prefix.as_deref(),
        "date",
    )));
    path.push('/');
    path.push_str(&escape_list_pages_html_attr(date));
    path
}

fn page_calendar_month_name(month: time::Month) -> &'static str {
    match month {
        time::Month::January => "January",
        time::Month::February => "February",
        time::Month::March => "March",
        time::Month::April => "April",
        time::Month::May => "May",
        time::Month::June => "June",
        time::Month::July => "July",
        time::Month::August => "August",
        time::Month::September => "September",
        time::Month::October => "October",
        time::Month::November => "November",
        time::Month::December => "December",
    }
}

fn render_page_calendar_error() -> String {
    format!(
        r#"<div class="error-block">{}</div>"#,
        escape_list_pages_html_text(PAGE_CALENDAR_CATEGORY_ERROR),
    )
}

fn render_page_calendar_module(
    arguments: &PageCalendarArguments,
    counts: &BTreeMap<i32, BTreeMap<u8, usize>>,
) -> String {
    let mut output = String::from("<div class=\"page-calendar-box\">\n\t\t\t<ul>\n");
    for (year, months) in counts.iter().rev() {
        let year_count = months.values().sum::<usize>();
        let year_date = year.to_string();
        let year_class = if arguments.selected_date.as_deref() == Some(year_date.as_str())
        {
            " class=\"selected\""
        } else {
            " "
        };
        output.push_str("\t\t\t\t\t<li");
        output.push_str(year_class);
        output.push_str(">\n\t\t\t\t<a href=\"");
        output.push_str(&page_calendar_path(arguments, &year_date));
        output.push_str("\">");
        output.push_str(&year.to_string());
        output.push_str(" (");
        output.push_str(&year_count.to_string());
        output.push_str(")</a>\n\t\t\t\t<ul>\n");

        for (month, count) in months.iter().rev() {
            let Some(month_name) = time::Month::try_from(*month)
                .ok()
                .map(page_calendar_month_name)
            else {
                continue;
            };
            let month_date = format!("{year}.{month}");
            let month_class =
                if arguments.selected_date.as_deref() == Some(month_date.as_str()) {
                    " class=\"selected\""
                } else {
                    " "
                };
            output.push_str("\t\t\t\t\t\t\t\t\t\t\t<li");
            output.push_str(month_class);
            output.push_str(">\n\t\t\t\t\t\t\t<a href=\"");
            output.push_str(&page_calendar_path(arguments, &month_date));
            output.push_str("\">");
            output.push_str(month_name);
            output.push_str(" (");
            output.push_str(&count.to_string());
            output.push_str(")</a>\n\t\t\t\t\t\t</li>\n");
        }

        output.push_str("\t\t\t\t\t\t\t\t\t</ul>\n\t\t\t</li>\n");
    }
    output.push_str("\t\t\t\t</ul>\n\t</div>");
    output
}

impl RenderService {
    pub(super) async fn expand_page_calendar_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        options: PageCalendarExpansionOptions<'_>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !PAGECALENDAR_MODULE_REGEX.is_match(&wikitext)
        {
            return Ok(wikitext);
        }

        let (Some(current_site_id), Some(current_page_id)) =
            (options.current_site_id, options.current_page_id)
        else {
            return Ok(wikitext);
        };

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let current_branch_tag = page_info
            .tags
            .iter()
            .find(|tag| tag.starts_with("branch-"))
            .map(Cow::as_ref);
        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        let mut counts_cache = PageCalendarCountsCache::default();

        for captures in PAGECALENDAR_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a PageCalendar capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }

            let head = captures.name("head").map_or("", |head| head.as_str());
            let Some(arguments) =
                parse_page_calendar_arguments(head, page_info, options.url)
            else {
                continue;
            };
            expanded.push_str(&wikitext[cursor..matched.start()]);
            let counts = counts_cache
                .get_or_init(arguments.categories.clone(), || {
                    Self::load_page_calendar_counts(
                        ctx,
                        current_site_id,
                        current_page_id,
                        current_branch_tag,
                        &arguments.categories,
                    )
                })
                .await?;
            match counts {
                Some(counts) => {
                    expanded.push_str(&compat_html.push_block_html(
                        render_page_calendar_module(&arguments, &counts),
                    ));
                }
                None => {
                    expanded.push_str(
                        &compat_html.push_block_html(render_page_calendar_error()),
                    );
                }
            }
            cursor = matched.end();
        }

        if cursor == 0 {
            return Ok(wikitext);
        }
        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn load_page_calendar_counts(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        _current_page_id: i64,
        current_branch_tag: Option<&str>,
        categories: &PageCalendarCategorySelector,
    ) -> Result<Option<BTreeMap<i32, BTreeMap<u8, usize>>>> {
        let make_error =
            || Error::new("failed to render PageCalendar module", ErrorType::Render);
        let txn = ctx.transaction();
        let category_ids = match categories {
            PageCalendarCategorySelector::All => None,
            PageCalendarCategorySelector::Names(names) => {
                let requested = names
                    .iter()
                    .map(|name| name.trim())
                    .filter(|name| !name.is_empty())
                    .collect::<BTreeSet<_>>();
                if requested.is_empty() {
                    return Ok(None);
                }

                let mut values = Vec::<Value>::with_capacity(requested.len() + 1);
                values.push(current_site_id.into());
                values.extend(requested.iter().map(|name| (*name).into()));
                let placeholders = (2..(requested.len() + 2))
                    .map(|index| format!("${index}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let categories = PageCalendarCategoryRow::find_by_statement(
                    Statement::from_sql_and_values(
                        txn.get_database_backend(),
                        format!(
                            "SELECT category_id, slug \
                             FROM page_category \
                             WHERE site_id = $1 \
                               AND slug IN ({placeholders})",
                        ),
                        values,
                    ),
                )
                .all(txn)
                .await
                .or_raise(make_error)?;
                let found = categories
                    .iter()
                    .map(|category| category.slug.as_str())
                    .collect::<BTreeSet<_>>();
                if found.len() != requested.len() {
                    return Ok(None);
                }
                Some(
                    categories
                        .into_iter()
                        .map(|category| category.category_id)
                        .collect::<Vec<_>>(),
                )
            }
        };

        let mut values = vec![current_site_id.into()];
        let category_filter = if let Some(category_ids) = &category_ids {
            values.extend(category_ids.iter().copied().map(Value::from));
            let placeholders = (2..(category_ids.len() + 2))
                .map(|index| format!("${index}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!(" AND p.page_category_id IN ({placeholders})")
        } else {
            String::new()
        };
        let statement = Statement::from_sql_and_values(
            txn.get_database_backend(),
            format!(
                "SELECT p.page_id, p.page_category_id, p.created_at, pr.tags \
                 FROM page p \
                 JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
                 WHERE p.site_id = $1 \
                   AND p.deleted_at IS NULL \
                   {category_filter}",
            ),
            values,
        );
        let pages = PageCalendarPageRow::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)?;
        let mut counts = BTreeMap::<i32, BTreeMap<u8, usize>>::new();
        let mut category_permissions = HashMap::new();
        for page in pages {
            let can_view = if let Some(can_view) =
                category_permissions.get(&page.page_category_id)
            {
                *can_view
            } else {
                let can_view = PermissionService::check_user_can(
                    ctx,
                    &CheckPermissionContext {
                        user_id: None,
                        site_id: current_site_id,
                        page_reference: Some(Reference::Id(page.page_id)),
                    },
                    Permission {
                        resource_type: Resource::Page,
                        resource_category: Some(Reference::Id(page.page_category_id)),
                        action: Action::View,
                    },
                )
                .await
                .or_raise(make_error)?;
                category_permissions.insert(page.page_category_id, can_view);
                can_view
            };
            if !can_view {
                continue;
            }

            if let Some(branch_tag) = current_branch_tag
                && !page.tags.iter().any(|tag| tag == branch_tag)
            {
                continue;
            }
            let month = u8::from(page.created_at.month());
            *counts
                .entry(page.created_at.year())
                .or_default()
                .entry(month)
                .or_default() += 1;
        }

        Ok(Some(counts))
    }
}

#[cfg(test)]
mod page_calendar_tests {
    use std::cell::Cell;

    use super::{PageCalendarCategorySelector, PageCalendarCountsCache};

    #[tokio::test]
    async fn repeated_page_calendar_modules_reuse_the_same_counts() {
        let loads = Cell::new(0);
        let mut cache = PageCalendarCountsCache::default();
        let category = PageCalendarCategorySelector::All;

        let first = cache
            .get_or_init(category.clone(), || async {
                loads.set(loads.get() + 1);
                Ok(Some(Default::default()))
            })
            .await
            .expect("the first calendar query should succeed");
        let second = cache
            .get_or_init(category, || async {
                panic!("a cached calendar result must not query again");
                #[allow(unreachable_code)]
                Ok(None)
            })
            .await
            .expect("the cached calendar query should succeed");

        assert_eq!(first, second);
        assert_eq!(loads.get(), 1);
    }
}
