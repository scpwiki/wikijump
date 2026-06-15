/*
 * bibliography.rs
 *
 * ftml - Library to parse Wikidot text
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

//! Structures managing bibliographic references.
//!
//! As an extension from Wikidot, we permit multiple bibliographies
//! to appear on a page, allowing people to have multiple sets,
//! for instance on separate tabs on a page.
//!
//! The first reference found is the one used.

use super::Element;
use super::clone::{elements_to_owned, string_to_owned};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Bibliography<'t>(Vec<(Cow<'t, str>, Vec<Element<'t>>)>);

impl<'t> Bibliography<'t> {
    pub fn new() -> Self {
        Bibliography::default()
    }

    pub fn add(&mut self, label: Cow<'t, str>, elements: Vec<Element<'t>>) {
        // If the reference already exists, it is *not* overwritten.
        //
        // This maintains the invariant that the first reference with a given label,
        // across any bibliography, is the one which is used.
        if self.get(&label).is_some() {
            warn!("Duplicate reference in bibliography: {label}");
            return;
        }

        self.0.push((label, elements));
    }

    pub fn get(&self, label: &str) -> Option<(usize, &[Element<'t>])> {
        // References are maintained as a list, which means that searching
        // for a particular label is O(n), but this is fine as the number
        // of references is always going to be bounded. Even at 100 references
        // this would run at essentially the same speed.
        //
        // This also gives us free indexing based on this order, and the
        // order based on it, so we don't need a two-index map here.
        for (index, (ref_label, elements)) in self.0.iter().enumerate() {
            if label == ref_label {
                // Change from zero-indexing to one-indexing
                return Some((index + 1, elements));
            }
        }

        None
    }

    #[inline]
    pub fn slice(&self) -> &[(Cow<'t, str>, Vec<Element<'t>>)] {
        &self.0
    }

    pub fn to_owned(&self) -> Bibliography<'static> {
        Bibliography(
            self.0
                .iter()
                .map(|(label, elements)| {
                    (string_to_owned(label), elements_to_owned(elements))
                })
                .collect(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct BibliographyList<'t>(Vec<Bibliography<'t>>);

impl<'t> BibliographyList<'t> {
    pub fn new() -> Self {
        BibliographyList::default()
    }

    pub fn push(&mut self, bibliography: Bibliography<'t>) {
        self.0.push(bibliography);
    }

    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn next_index(&self) -> usize {
        self.0.len()
    }

    pub fn get_reference(&self, label: &str) -> Option<(usize, &[Element<'t>])> {
        for bibliography in &self.0 {
            // Find the first entry with the label, per the above invariant.
            let reference = bibliography.get(label);
            if reference.is_some() {
                return reference;
            }
        }

        None
    }

    pub fn get_bibliography(&self, index: usize) -> &Bibliography<'t> {
        &self.0[index]
    }

    pub fn to_owned(&self) -> BibliographyList<'static> {
        BibliographyList(self.0.iter().map(|b| b.to_owned()).collect())
    }
}
