//! Render-local analysis for a ListPages body template.

use crate::services::page_query::FoundPageFields;
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::LazyLock;

pub(in crate::services::render) static LISTPAGES_VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(
        r"%%(?P<name>[A-Za-z0-9_]+)(?:\{(?P<argument>[A-Za-z0-9_-]+)\})?(?:\((?P<length>[0-9]+)\))?(?:\|(?P<format>.*?))?%%",
    )
    .unwrap()
    });

const DEFAULT_LISTPAGES_TEMPLATE: &str =
    "+ %%title_linked%%\n\nby %%created_by_linked%% %%created_at%%\n\n%%summary%%";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::services::render) enum ListPagesOutputShape {
    Plain,
    NumberedRows,
    TableRows,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum ListPagesVariable {
    TitleLinked,
    Title,
    Slug,
    FullSlug,
    Link,
    CreatedBy,
    CreatedByLinked,
    CreatedByUnix,
    CreatedById,
    CreatedAt,
    UpdatedBy,
    UpdatedByUnix,
    UpdatedById,
    UpdatedAt,
    CommentedBy,
    CommentedAt,
    Rating,
    RatingVotes,
    RatingPercent,
    Comments,
    Tags,
    TagsLinked,
    HiddenTagsLinked,
    RawTags,
    Category,
    Size,
    SiteDomain,
    SiteTitle,
    SiteName,
    ParentFullname,
    ParentName,
    ParentCategory,
    ParentTitle,
    ParentTitleLinked,
    Revisions,
    Children,
    FormData,
    Content,
    Preview,
    Summary,
    Index,
    Total,
    Limit,
    TotalOrLimit,
}

impl ListPagesVariable {
    fn parse(name: &str, has_argument: bool) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "title_linked" | "linked_title" => Some(Self::TitleLinked),
            "title" => Some(Self::Title),
            "name" | "slug" | "page_name" => Some(Self::Slug),
            "fullname" | "full_slug" | "page_unix_name" | "full_page_name" => {
                Some(Self::FullSlug)
            }
            "link" => Some(Self::Link),
            "created_by" | "createdby" => Some(Self::CreatedBy),
            "created_by_linked" | "createdbylinked" | "author" => {
                Some(Self::CreatedByLinked)
            }
            // Only the evidenced spelling is accepted. The collapsed aliases in
            // this table were each observed live; an unobserved variant stays
            // literal rather than being guessed from a naming pattern.
            "created_by_unix" => Some(Self::CreatedByUnix),
            "created_by_id" => Some(Self::CreatedById),
            "created_at" | "createdat" | "date" => Some(Self::CreatedAt),
            "updated_by" | "updatedby" | "updated_by_linked" | "updatedbylinked"
            | "author_edited" | "user_edited" => Some(Self::UpdatedBy),
            "updated_by_unix" => Some(Self::UpdatedByUnix),
            "updated_by_id" => Some(Self::UpdatedById),
            "updated_at" | "updatedat" | "date_edited" => Some(Self::UpdatedAt),
            "commented_by"
            | "commentedby"
            | "commented_by_linked"
            | "commentedbylinked"
            | "commented_by_unix"
            | "commented_by_id" => Some(Self::CommentedBy),
            "commented_at" | "commentedat" => Some(Self::CommentedAt),
            "rating" => Some(Self::Rating),
            "rating_votes" | "ratingvotes" => Some(Self::RatingVotes),
            "rating_percent" => Some(Self::RatingPercent),
            "comments" => Some(Self::Comments),
            "tags" => Some(Self::Tags),
            "tags_linked" | "tagslinked" => Some(Self::TagsLinked),
            "_tags_linked" => Some(Self::HiddenTagsLinked),
            "_tags" => Some(Self::RawTags),
            "category" => Some(Self::Category),
            "size" => Some(Self::Size),
            "site_domain" => Some(Self::SiteDomain),
            "site_title" => Some(Self::SiteTitle),
            "site_name" => Some(Self::SiteName),
            "parent_fullname" => Some(Self::ParentFullname),
            "parent_name" => Some(Self::ParentName),
            "parent_category" => Some(Self::ParentCategory),
            "parent_title" => Some(Self::ParentTitle),
            "parent_title_linked" => Some(Self::ParentTitleLinked),
            "revisions" => Some(Self::Revisions),
            "children" => Some(Self::Children),
            "form_data" | "form_raw" | "form_label" | "form_hint" if has_argument => {
                Some(Self::FormData)
            }
            "content" | "text" | "long" | "body" => Some(Self::Content),
            "preview" => Some(Self::Preview),
            "summary" | "first_paragraph" | "description" | "short" => {
                Some(Self::Summary)
            }
            "index" => Some(Self::Index),
            "total" => Some(Self::Total),
            "limit" => Some(Self::Limit),
            "total_or_limit" => Some(Self::TotalOrLimit),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ListPagesVariables(u64);

impl ListPagesVariables {
    fn insert(&mut self, variable: ListPagesVariable) {
        self.0 |= 1_u64 << variable as u8;
    }

    fn contains(self, variable: ListPagesVariable) -> bool {
        self.0 & (1_u64 << variable as u8) != 0
    }

    fn intersects(self, variables: &[ListPagesVariable]) -> bool {
        variables
            .iter()
            .copied()
            .any(|variable| self.contains(variable))
    }
}

#[derive(Debug)]
pub(in crate::services::render) struct ListPagesTemplatePlan {
    body: String,
    sections: ListPagesSections,
    variables: ListPagesVariables,
    fields: FoundPageFields,
    content_sections: BTreeSet<Option<usize>>,
    output_shape: ListPagesOutputShape,
    rating_only: bool,
    #[cfg(test)]
    variable_traversals: usize,
}

/// The `[[head]]`, `[[body]]`, and `[[foot]]` split of a ListPages template.
///
/// Wikidot emits the head once, the body once per selected row, and the foot
/// once, all inside the result wrapper. A template without any marker is one
/// undivided per-row body.
#[derive(Debug, Default, PartialEq, Eq)]
struct ListPagesSections {
    head: Option<String>,
    foot: Option<String>,
}

/// Splits a template into its once-emitted sections and its per-row body.
///
/// Returns `None` for a shape whose live output is uncaptured: a repeated
/// marker, a marker that never closes, or a `[[head]]`/`[[foot]]` without the
/// `[[body]]` that would separate them from the row template.
fn split_list_pages_sections(body: &str) -> Option<(ListPagesSections, String)> {
    let mut sections = ListPagesSections::default();
    let mut row_body = None;

    for (name, slot) in [("head", 0), ("body", 1), ("foot", 2)] {
        let open = format!("[[{name}]]");
        let close = format!("[[/{name}]]");
        let mut matches = body.match_indices(&open);
        let Some((open_start, _)) = matches.next() else {
            continue;
        };
        if matches.next().is_some() {
            return None;
        }
        if body.matches(&close).count() != 1 {
            return None;
        }
        let content_start = open_start + open.len();
        let close_start = body[content_start..].find(&close)? + content_start;
        let content = body[content_start..close_start].trim().to_owned();
        match slot {
            0 => sections.head = Some(content),
            1 => row_body = Some(content),
            _ => sections.foot = Some(content),
        }
    }

    match row_body {
        Some(row_body) => Some((sections, row_body)),
        // A head or foot with no body has no evidenced row template to pair it
        // with, so the module is left alone rather than guessed at.
        None if sections == ListPagesSections::default() => {
            Some((sections, body.to_owned()))
        }
        None => None,
    }
}

impl ListPagesTemplatePlan {
    pub(in crate::services::render) fn compile(body: &str) -> Option<Self> {
        let (sections, body) = split_list_pages_sections(body)?;
        let body = match body.trim() {
            "" if sections == ListPagesSections::default() => DEFAULT_LISTPAGES_TEMPLATE,
            body => body,
        };
        let mut variables = ListPagesVariables::default();
        let mut content_sections = BTreeSet::new();
        let mut variable_count = 0;
        let mut rating_only = true;

        for captures in LISTPAGES_VARIABLE_REGEX.captures_iter(body) {
            let variable = ListPagesVariable::parse(
                &captures["name"],
                captures.name("argument").is_some(),
            )?;
            variable_count += 1;
            rating_only &= variable == ListPagesVariable::Rating;
            variables.insert(variable);
            if variable == ListPagesVariable::Content {
                content_sections.insert(
                    captures
                        .name("argument")
                        .and_then(|matched| matched.as_str().parse().ok()),
                );
            }
        }

        // A row variable in a once-emitted section has no row to read from, and
        // Wikidot's output for that shape is uncaptured.
        if [sections.head.as_deref(), sections.foot.as_deref()]
            .into_iter()
            .flatten()
            .any(|section| LISTPAGES_VARIABLE_REGEX.is_match(section))
        {
            return None;
        }

        Some(Self {
            body: body.to_owned(),
            sections,
            variables,
            fields: found_page_fields(variables),
            content_sections,
            output_shape: output_shape(body),
            rating_only: variable_count > 0 && rating_only,
            #[cfg(test)]
            variable_traversals: 1,
        })
    }

    pub(in crate::services::render) fn body(&self) -> &str {
        &self.body
    }

    /// The section emitted once before the rows, if the template declares one.
    pub(in crate::services::render) fn head_section(&self) -> Option<&str> {
        self.sections.head.as_deref()
    }

    /// The section emitted once after the rows, if the template declares one.
    pub(in crate::services::render) fn foot_section(&self) -> Option<&str> {
        self.sections.foot.as_deref()
    }

    /// Whether the template splits itself into once-emitted sections.
    pub(in crate::services::render) fn has_sections(&self) -> bool {
        self.sections != ListPagesSections::default()
    }

    pub(in crate::services::render) fn fields(&self) -> FoundPageFields {
        self.fields.clone()
    }

    pub(in crate::services::render) fn output_shape(&self) -> ListPagesOutputShape {
        self.output_shape
    }

    pub(in crate::services::render) fn uses_created_by(&self) -> bool {
        self.variables.intersects(&[
            ListPagesVariable::CreatedBy,
            ListPagesVariable::CreatedByLinked,
            ListPagesVariable::CreatedByUnix,
            ListPagesVariable::CreatedById,
        ])
    }

    pub(in crate::services::render) fn uses_created_by_unix(&self) -> bool {
        self.variables.contains(ListPagesVariable::CreatedByUnix)
    }

    pub(in crate::services::render) fn uses_created_at(&self) -> bool {
        self.variables.contains(ListPagesVariable::CreatedAt)
    }

    pub(in crate::services::render) fn uses_updated_by(&self) -> bool {
        self.variables.intersects(&[
            ListPagesVariable::UpdatedBy,
            ListPagesVariable::UpdatedByUnix,
            ListPagesVariable::UpdatedById,
        ])
    }

    pub(in crate::services::render) fn uses_updated_at(&self) -> bool {
        self.variables.contains(ListPagesVariable::UpdatedAt)
    }

    pub(in crate::services::render) fn uses_comments(&self) -> bool {
        self.variables.contains(ListPagesVariable::Comments)
    }

    pub(in crate::services::render) fn uses_commented_by(&self) -> bool {
        self.variables.contains(ListPagesVariable::CommentedBy)
    }

    pub(in crate::services::render) fn uses_commented_at(&self) -> bool {
        self.variables.contains(ListPagesVariable::CommentedAt)
    }

    pub(in crate::services::render) fn uses_rating_votes(&self) -> bool {
        self.variables.contains(ListPagesVariable::RatingVotes)
    }

    pub(in crate::services::render) fn uses_rating(&self) -> bool {
        self.variables.contains(ListPagesVariable::Rating)
    }

    pub(in crate::services::render) fn uses_rating_percent(&self) -> bool {
        self.variables.contains(ListPagesVariable::RatingPercent)
    }

    pub(in crate::services::render) fn uses_content(&self) -> bool {
        self.variables.intersects(&[
            ListPagesVariable::Content,
            ListPagesVariable::Preview,
            ListPagesVariable::Summary,
        ])
    }

    pub(in crate::services::render) fn uses_size(&self) -> bool {
        self.variables.contains(ListPagesVariable::Size)
    }

    pub(in crate::services::render) fn uses_site_domain(&self) -> bool {
        self.variables.contains(ListPagesVariable::SiteDomain)
    }

    pub(in crate::services::render) fn uses_site_title(&self) -> bool {
        self.variables.contains(ListPagesVariable::SiteTitle)
    }

    pub(in crate::services::render) fn uses_parent_metadata(&self) -> bool {
        self.variables.intersects(&[
            ListPagesVariable::ParentFullname,
            ListPagesVariable::ParentName,
            ListPagesVariable::ParentCategory,
            ListPagesVariable::ParentTitle,
            ListPagesVariable::ParentTitleLinked,
        ])
    }

    pub(in crate::services::render) fn uses_total(&self) -> bool {
        self.variables.contains(ListPagesVariable::Total)
    }

    pub(in crate::services::render) fn uses_revisions(&self) -> bool {
        self.variables.contains(ListPagesVariable::Revisions)
    }

    pub(in crate::services::render) fn uses_children(&self) -> bool {
        self.variables.contains(ListPagesVariable::Children)
    }

    pub(in crate::services::render) fn content_sections(
        &self,
    ) -> &BTreeSet<Option<usize>> {
        &self.content_sections
    }

    pub(in crate::services::render) fn uses_data_form(&self) -> bool {
        self.variables.contains(ListPagesVariable::FormData)
    }

    pub(in crate::services::render) fn uses_only_rating(&self) -> bool {
        self.rating_only
    }

    #[cfg(test)]
    fn variable_traversals(&self) -> usize {
        self.variable_traversals
    }
}

fn found_page_fields(variables: ListPagesVariables) -> FoundPageFields {
    let created_by = variables.intersects(&[
        ListPagesVariable::CreatedBy,
        ListPagesVariable::CreatedByLinked,
        ListPagesVariable::CreatedByUnix,
        ListPagesVariable::CreatedById,
    ]);
    let rating_votes = variables.contains(ListPagesVariable::RatingVotes);
    FoundPageFields {
        title: true,
        slug: true,
        page_category_id: true,
        created_by,
        created_at: variables.contains(ListPagesVariable::CreatedAt),
        tags: variables.intersects(&[
            ListPagesVariable::Tags,
            ListPagesVariable::TagsLinked,
            ListPagesVariable::HiddenTagsLinked,
            ListPagesVariable::RawTags,
        ]),
        updated_by: variables.intersects(&[
            ListPagesVariable::UpdatedBy,
            ListPagesVariable::UpdatedByUnix,
            ListPagesVariable::UpdatedById,
        ]),
        updated_at: variables.contains(ListPagesVariable::UpdatedAt),
        score: variables.contains(ListPagesVariable::Rating)
            || variables.contains(ListPagesVariable::RatingPercent)
            || rating_votes,
        ..Default::default()
    }
}

fn output_shape(body: &str) -> ListPagesOutputShape {
    let mut saw_nonempty = false;
    let mut table_rows = true;
    let mut numbered_rows = false;
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        saw_nonempty = true;
        let trimmed = line.trim();
        table_rows &= trimmed.ends_with("||")
            && (trimmed.starts_with("||=") || trimmed.starts_with("||~"));
        numbered_rows |= line.trim_start_matches(' ').starts_with("# ");
    }

    if saw_nonempty && table_rows {
        ListPagesOutputShape::TableRows
    } else if numbered_rows {
        ListPagesOutputShape::NumberedRows
    } else {
        ListPagesOutputShape::Plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_aliases_into_field_and_dependency_requirements_once() {
        let body = concat!(
            "%%createdbylinked%% %%date%% %%tagslinked%% %%_tags_linked%% %%updatedby%% ",
            "%%updatedat%% %%date_edited%% %%ratingvotes%% %%comments%% %%commentedby%% ",
            "%%commentedat%% %%content%% %%form_raw{status}%% %%size%% %%created_by_unix%% ",
            "%%site_domain%% %%parent_fullname%% %%revisions%% %%children%%",
        );
        let plan = ListPagesTemplatePlan::compile(body).expect("aliases should compile");

        assert!(plan.uses_created_by());
        assert!(plan.uses_created_by_unix());
        assert!(plan.uses_created_at());
        assert!(plan.uses_updated_by());
        assert!(plan.uses_updated_at());
        assert!(plan.uses_rating_votes());
        assert!(plan.uses_comments());
        assert!(plan.uses_commented_by());
        assert!(plan.uses_commented_at());
        assert!(plan.uses_content());
        assert!(plan.uses_size());
        assert!(plan.uses_site_domain());
        assert!(plan.uses_parent_metadata());
        assert!(plan.uses_revisions());
        assert!(plan.uses_children());
        assert_eq!(plan.content_sections(), &BTreeSet::from([None]));
        assert!(plan.uses_data_form());
        assert_eq!(plan.variable_traversals(), 1);
        assert_eq!(
            plan.fields(),
            FoundPageFields {
                title: true,
                slug: true,
                page_category_id: true,
                tags: true,
                created_at: true,
                created_by: true,
                updated_at: true,
                updated_by: true,
                score: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn rejects_unknown_and_argumentless_form_variables() {
        assert!(ListPagesTemplatePlan::compile("%%unsupported%%").is_none());
        assert!(ListPagesTemplatePlan::compile("%%createdbyunix%%").is_none());
        assert!(ListPagesTemplatePlan::compile("%%form_data%%").is_none());
        assert!(ListPagesTemplatePlan::compile("%%form_raw%%").is_none());
    }

    #[test]
    fn records_distinct_content_sections_during_compilation() {
        let plan = ListPagesTemplatePlan::compile(
            "%%content{2}%% %%content{4}%% %%content{2}%% %%content%%",
        )
        .expect("content sections should compile");

        assert_eq!(
            plan.content_sections(),
            &BTreeSet::from([None, Some(2), Some(4)]),
        );
    }

    #[test]
    fn accepts_every_supported_alias_and_variable_suffix() {
        for name in [
            "title_linked",
            "linked_title",
            "title",
            "name",
            "slug",
            "page_name",
            "page_unix_name",
            "full_page_name",
            "fullname",
            "full_slug",
            "link",
            "created_by",
            "createdby",
            "created_by_linked",
            "createdbylinked",
            "created_by_unix",
            "created_by_id",
            "author",
            "created_at",
            "createdat",
            "date",
            "updated_by",
            "updatedby",
            "updated_by_linked",
            "updatedbylinked",
            "author_edited",
            "user_edited",
            "updated_by_unix",
            "updated_by_id",
            "updated_at",
            "updatedat",
            "date_edited",
            "commented_by",
            "commentedby",
            "commented_by_linked",
            "commentedbylinked",
            "commented_by_unix",
            "commented_by_id",
            "commented_at",
            "commentedat",
            "rating",
            "rating_votes",
            "ratingvotes",
            "comments",
            "tags",
            "tags_linked",
            "_tags_linked",
            "category",
            "tagslinked",
            "_tags",
            "site_domain",
            "site_title",
            "site_name",
            "parent_fullname",
            "parent_name",
            "parent_category",
            "parent_title",
            "parent_title_linked",
            "size",
            "children",
            "rating_percent",
            "revisions",
            "content",
            "text",
            "long",
            "body",
            "preview",
            "summary",
            "first_paragraph",
            "description",
            "short",
            "index",
            "total",
            "limit",
            "total_or_limit",
        ] {
            let body = format!("%%{name}|format suffix%%");
            assert!(
                ListPagesTemplatePlan::compile(&body).is_some(),
                "unsupported alias: {name}",
            );
        }
        for name in ["form_data", "form_raw", "form_label", "form_hint"] {
            assert!(
                ListPagesTemplatePlan::compile(&format!("%%{name}{{field-name}}%%"))
                    .is_some(),
            );
        }
    }

    #[test]
    fn classifies_rating_only_and_output_shapes() {
        let rating = ListPagesTemplatePlan::compile("article [+%%rating%%]")
            .expect("rating body should compile");
        assert!(rating.uses_only_rating());
        assert_eq!(rating.output_shape(), ListPagesOutputShape::Plain);

        let numbered = ListPagesTemplatePlan::compile("# %%title%%\n# %%rating%%")
            .expect("numbered body should compile");
        assert!(!numbered.uses_only_rating());
        assert_eq!(numbered.output_shape(), ListPagesOutputShape::NumberedRows);

        let table = ListPagesTemplatePlan::compile("||~ %%title%% ||\n||= %%rating%% ||")
            .expect("table body should compile");
        assert_eq!(table.output_shape(), ListPagesOutputShape::TableRows);

        let wikidot_cells = ListPagesTemplatePlan::compile("|| %%title%% ||")
            .expect("ordinary cells should remain supported");
        assert_eq!(wikidot_cells.output_shape(), ListPagesOutputShape::Plain);
    }

    #[test]
    fn synthetic_ralliston_shape_uses_one_variable_traversal_per_template() {
        let plans = (0..191)
            .map(|index| {
                ListPagesTemplatePlan::compile(&format!(
                    "unique article {index} [+%%rating%%]"
                ))
                .expect("rating template should compile")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            plans
                .iter()
                .map(ListPagesTemplatePlan::variable_traversals)
                .sum::<usize>(),
            191,
        );
        assert!(plans.iter().all(ListPagesTemplatePlan::uses_only_rating));
    }
}

#[cfg(test)]
mod section_tests {
    use super::*;

    #[test]
    fn splits_the_evidenced_head_body_foot_template() {
        // The G59 probe template, whose live output was one head, four body
        // rows, and one foot inside the result wrapper.
        let plan = ListPagesTemplatePlan::compile(
            "[[head]]G59_HEAD;[[/head]][[body]]G59_BODY=%%name%%;[[/body]][[foot]]G59_FOOT;[[/foot]]",
        )
        .expect("the sectioned template should compile");

        assert_eq!(plan.head_section(), Some("G59_HEAD;"));
        assert_eq!(plan.body(), "G59_BODY=%%name%%;");
        assert_eq!(plan.foot_section(), Some("G59_FOOT;"));
        assert!(plan.has_sections());
    }

    #[test]
    fn an_unsectioned_template_is_one_per_row_body() {
        let plan = ListPagesTemplatePlan::compile("%%title_linked%%")
            .expect("an ordinary template should compile");

        assert_eq!(plan.head_section(), None);
        assert_eq!(plan.foot_section(), None);
        assert_eq!(plan.body(), "%%title_linked%%");
        assert!(!plan.has_sections());
    }

    #[test]
    fn trims_module_and_section_boundary_whitespace_before_rows_are_combined() {
        let plan = ListPagesTemplatePlan::compile(
            "\n  [[head]]\n  H\n  [[/head]]\n  [[body]]\n  B=%%name%%\n  [[/body]]\n  [[foot]]\n  F\n  [[/foot]]\n",
        )
        .expect("boundary whitespace should not split combined ListPages output");

        assert_eq!(plan.head_section(), Some("H"));
        assert_eq!(plan.body(), "B=%%name%%");
        assert_eq!(plan.foot_section(), Some("F"));
    }

    #[test]
    fn head_or_foot_without_a_body_has_no_row_template() {
        assert!(ListPagesTemplatePlan::compile("[[head]]H[[/head]]%%name%%").is_none());
        assert!(ListPagesTemplatePlan::compile("[[foot]]F[[/foot]]%%name%%").is_none());
    }

    #[test]
    fn repeated_or_unclosed_markers_do_not_compile() {
        assert!(
            ListPagesTemplatePlan::compile("[[body]]A[[/body]][[body]]B[[/body]]")
                .is_none(),
            "a repeated marker has no evidenced precedence",
        );
        assert!(
            ListPagesTemplatePlan::compile("[[body]]A").is_none(),
            "an unclosed marker has no evidenced extent",
        );
        assert!(
            ListPagesTemplatePlan::compile(
                "[[head]]H[[/head]][[body]]A[[/body]][[foot]]F"
            )
            .is_none(),
        );
    }

    #[test]
    fn a_row_variable_in_a_once_emitted_section_does_not_compile() {
        assert!(
            ListPagesTemplatePlan::compile(
                "[[head]]%%title%%[[/head]][[body]]%%name%%[[/body]]"
            )
            .is_none(),
            "a head emitted once has no row to read a page variable from",
        );
    }
}
