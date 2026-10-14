//! Restoring compatibility-protected text inside parsed link targets.
//!
//! Text that FTML must not interpret (unbound `{$variables}`, stray `##`
//! color markers, ...) is replaced by opaque markers before parsing and put
//! back into the rendered HTML afterwards. A marker inside a link target
//! cannot be put back that way: FTML lowercases and slug-normalizes page
//! targets, and the link rows Deepwell records come from the parsed targets,
//! not from the restored HTML. Wikidot itself never sees a marker; it builds
//! the link from the authored text, so `[[[{$previous-url} | ...]]]` links to
//! `/previous-url`.
//!
//! Restoring the authored text into the parsed link locations before page
//! existence is resolved and HTML is rendered lets FTML's normal link
//! handling produce the same target, class, and recorded link as for any
//! other authored target.

use super::text_fragments::CompatTextFragments;
use ftml::data::PageRef;
use ftml::tree::{
    Container, DefinitionListItem, Element, LinkLocation, ListItem, PartialElement,
    SyntaxTree,
};
use std::borrow::Cow;
use wikidot_normalize::normalize;

/// Nesting deeper than this is left as parsed; FTML bounds pathological
/// nesting itself, and an unrestored marker only reproduces the old output.
const MAX_DEPTH: usize = 512;

pub(in crate::services::render) fn restore_compat_text_in_link_targets(
    tree: &mut SyntaxTree<'_>,
    fragments: &CompatTextFragments,
) {
    if fragments.is_empty() {
        return;
    }
    restore_elements(&mut tree.elements, fragments, 0);
    restore_elements(&mut tree.table_of_contents, fragments, 0);
    for footnote in &mut tree.footnotes {
        restore_elements(footnote, fragments, 0);
    }
}

fn restore_elements(
    elements: &mut [Element<'_>],
    fragments: &CompatTextFragments,
    depth: usize,
) {
    if depth > MAX_DEPTH {
        return;
    }
    for element in elements {
        restore_element(element, fragments, depth);
    }
}

fn restore_element(
    element: &mut Element<'_>,
    fragments: &CompatTextFragments,
    depth: usize,
) {
    let next = depth + 1;
    match element {
        Element::Link { link, .. } => restore_location(link, fragments),
        Element::Image {
            link: Some(link), ..
        } => restore_location(link, fragments),
        Element::Container(container) => {
            let ctype = container.ctype();
            let attributes = container.attributes().clone();
            let placeholder = Container::new(ctype, Vec::new(), attributes.clone());
            let mut children: Vec<Element<'_>> =
                std::mem::replace(container, placeholder).into();
            restore_elements(&mut children, fragments, next);
            *container = Container::new(ctype, children, attributes);
        }
        Element::Table(table) => {
            for row in &mut table.rows {
                for cell in &mut row.cells {
                    restore_elements(&mut cell.elements, fragments, next);
                }
            }
        }
        Element::TabView(tabs) => {
            for tab in tabs {
                restore_elements(&mut tab.elements, fragments, next);
            }
        }
        Element::Anchor { elements, .. }
        | Element::Collapsible { elements, .. }
        | Element::Color { elements, .. }
        | Element::Include { elements, .. } => {
            restore_elements(elements, fragments, next);
        }
        Element::List { items, .. } => {
            for item in items {
                restore_list_item(item, fragments, next);
            }
        }
        Element::DefinitionList(items) => {
            for DefinitionListItem {
                key_elements,
                value_elements,
                ..
            } in items
            {
                restore_elements(key_elements, fragments, next);
                restore_elements(value_elements, fragments, next);
            }
        }
        Element::Partial(partial) => match partial {
            PartialElement::ListItem(item) => restore_list_item(item, fragments, next),
            PartialElement::TableRow(row) => {
                for cell in &mut row.cells {
                    restore_elements(&mut cell.elements, fragments, next);
                }
            }
            PartialElement::TableCell(cell) => {
                restore_elements(&mut cell.elements, fragments, next);
            }
            PartialElement::Tab(tab) => {
                restore_elements(&mut tab.elements, fragments, next)
            }
            _ => {}
        },
        _ => {}
    }
}

fn restore_list_item(
    item: &mut ListItem<'_>,
    fragments: &CompatTextFragments,
    depth: usize,
) {
    match item {
        ListItem::Elements { elements, .. } => {
            restore_elements(elements, fragments, depth)
        }
        ListItem::SubList { element } => restore_element(element, fragments, depth),
    }
}

fn restore_location(location: &mut LinkLocation<'_>, fragments: &CompatTextFragments) {
    match location {
        LinkLocation::Page(PageRef { site, page, extra }) => {
            if fragments.contains_marker(page) {
                // FTML stores page targets already slug-normalized, so the
                // authored text is normalized by the same Wikidot rules.
                let mut restored = fragments.restore_ignoring_case(page);
                normalize(&mut restored);
                *page = restored;
            }
            if let Some(site) = site {
                restore_string(site, fragments);
            }
            if let Some(extra) = extra {
                restore_string(extra, fragments);
            }
        }
        LinkLocation::Url(url) => {
            if fragments.contains_marker(url) {
                *url = Cow::Owned(fragments.restore_ignoring_case(url));
            }
        }
    }
}

fn restore_string(value: &mut String, fragments: &CompatTextFragments) {
    if fragments.contains_marker(value) {
        *value = fragments.restore_ignoring_case(value);
    }
}
