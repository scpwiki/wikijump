//! Render-local analysis for a ListPages body template.

use crate::services::page_query::FoundPageFields;
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::LazyLock;

pub(in crate::services::render) static LISTPAGES_VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(
        r"%%(?P<name>[A-Za-z0-9_]+)(?:\{(?P<argument>[A-Za-z0-9_-]+)\})?(?:\|(?P<format>.*?))?%%",
    )
    .unwrap()
    });

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
    CreatedAt,
    UpdatedBy,
    UpdatedAt,
    CommentedBy,
    CommentedAt,
    Rating,
    RatingVotes,
    Comments,
    Tags,
    TagsLinked,
    HiddenTagsLinked,
    RawTags,
    Category,
    Size,
    SiteDomain,
    ParentFullname,
    Revisions,
    EmptyCompatField,
    FormData,
    Content,
    Index,
    Total,
    Limit,
}

impl ListPagesVariable {
    fn parse(name: &str, has_argument: bool) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "title_linked" | "linked_title" => Some(Self::TitleLinked),
            "title" => Some(Self::Title),
            "name" | "slug" | "page_unix_name" => Some(Self::Slug),
            "fullname" | "full_slug" => Some(Self::FullSlug),
            "link" => Some(Self::Link),
            "created_by" | "createdby" => Some(Self::CreatedBy),
            "created_by_linked" | "createdbylinked" | "author" => {
                Some(Self::CreatedByLinked)
            }
            // Only the evidenced spelling is accepted. The collapsed aliases in
            // this table were each observed live; an unobserved variant stays
            // literal rather than being guessed from a naming pattern.
            "created_by_unix" => Some(Self::CreatedByUnix),
            "created_at" | "createdat" | "date" => Some(Self::CreatedAt),
            "updated_by" | "updatedby" | "updated_by_linked" | "updatedbylinked" => {
                Some(Self::UpdatedBy)
            }
            "updated_at" | "updatedat" | "date_edited" => Some(Self::UpdatedAt),
            "commented_by"
            | "commentedby"
            | "commented_by_linked"
            | "commentedbylinked" => Some(Self::CommentedBy),
            "commented_at" | "commentedat" => Some(Self::CommentedAt),
            "rating" => Some(Self::Rating),
            "rating_votes" | "ratingvotes" => Some(Self::RatingVotes),
            "comments" => Some(Self::Comments),
            "tags" => Some(Self::Tags),
            "tags_linked" | "tagslinked" => Some(Self::TagsLinked),
            "_tags_linked" => Some(Self::HiddenTagsLinked),
            "_tags" => Some(Self::RawTags),
            "category" => Some(Self::Category),
            "size" => Some(Self::Size),
            "site_domain" => Some(Self::SiteDomain),
            "parent_fullname" => Some(Self::ParentFullname),
            "revisions" => Some(Self::Revisions),
            "children" | "rating_percent" => Some(Self::EmptyCompatField),
            "form_data" | "form_raw" if has_argument => Some(Self::FormData),
            "content" => Some(Self::Content),
            "index" => Some(Self::Index),
            "total" => Some(Self::Total),
            "limit" => Some(Self::Limit),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ListPagesVariables(u32);

impl ListPagesVariables {
    fn insert(&mut self, variable: ListPagesVariable) {
        self.0 |= 1 << variable as u8;
    }

    fn contains(self, variable: ListPagesVariable) -> bool {
        self.0 & (1 << variable as u8) != 0
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
    variables: ListPagesVariables,
    fields: FoundPageFields,
    content_sections: BTreeSet<Option<usize>>,
    output_shape: ListPagesOutputShape,
    rating_only: bool,
    #[cfg(test)]
    variable_traversals: usize,
}

impl ListPagesTemplatePlan {
    pub(in crate::services::render) fn compile(body: &str) -> Option<Self> {
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

        Some(Self {
            body: body.to_owned(),
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
        ])
    }

    pub(in crate::services::render) fn uses_created_by_unix(&self) -> bool {
        self.variables.contains(ListPagesVariable::CreatedByUnix)
    }

    pub(in crate::services::render) fn uses_created_at(&self) -> bool {
        self.variables.contains(ListPagesVariable::CreatedAt)
    }

    pub(in crate::services::render) fn uses_updated_by(&self) -> bool {
        self.variables.contains(ListPagesVariable::UpdatedBy)
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

    pub(in crate::services::render) fn uses_content(&self) -> bool {
        self.variables.contains(ListPagesVariable::Content)
    }

    pub(in crate::services::render) fn uses_size(&self) -> bool {
        self.variables.contains(ListPagesVariable::Size)
    }

    pub(in crate::services::render) fn uses_site_domain(&self) -> bool {
        self.variables.contains(ListPagesVariable::SiteDomain)
    }

    pub(in crate::services::render) fn uses_parent_fullname(&self) -> bool {
        self.variables.contains(ListPagesVariable::ParentFullname)
    }

    pub(in crate::services::render) fn uses_revisions(&self) -> bool {
        self.variables.contains(ListPagesVariable::Revisions)
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
        updated_by: variables.contains(ListPagesVariable::UpdatedBy),
        updated_at: variables.contains(ListPagesVariable::UpdatedAt),
        score: variables.contains(ListPagesVariable::Rating) || rating_votes,
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
            "%%site_domain%% %%parent_fullname%% %%revisions%%",
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
        assert!(plan.uses_parent_fullname());
        assert!(plan.uses_revisions());
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
            "page_unix_name",
            "fullname",
            "full_slug",
            "link",
            "created_by",
            "createdby",
            "created_by_linked",
            "createdbylinked",
            "created_by_unix",
            "author",
            "created_at",
            "createdat",
            "date",
            "updated_by",
            "updatedby",
            "updated_by_linked",
            "updatedbylinked",
            "updated_at",
            "updatedat",
            "date_edited",
            "commented_by",
            "commentedby",
            "commented_by_linked",
            "commentedbylinked",
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
            "parent_fullname",
            "size",
            "children",
            "rating_percent",
            "revisions",
            "content",
            "index",
            "total",
            "limit",
        ] {
            let body = format!("%%{name}|format suffix%%");
            assert!(
                ListPagesTemplatePlan::compile(&body).is_some(),
                "unsupported alias: {name}",
            );
        }
        for name in ["form_data", "form_raw"] {
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
