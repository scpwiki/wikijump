/*
 * tree/bibliography.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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

use crate::tree::Element;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Bibliography<'t> {
    references: HashMap<Cow<'t, str>, (u32, Vec<Element<'t>>)>,
    last_index: u32,
}

impl<'t> Bibliography<'t> {
    pub fn new() -> Self {
        Bibliography {
            references: HashMap::new(),
            last_index: 1,
        }
    }

    pub fn add(&mut self, label: Cow<'t, str>, elements: Vec<Element<'t>>) {
        if self.references.get(&label).is_some() {
            // If the reference already exists, it is *not* overwritten.
            //
            // This maintains the invariant that the first reference with a given label,
            // across any bibliography, is the one which is used.
            return;
        }

        let index = self.last_index;
        self.references.insert(label, (index, elements));
        self.last_index += 1;
    }

    #[inline]
    pub fn get(&self, label: &str) -> Option<(u32, &[Element<'t>])> {
        self.references
            .get(label)
            .map(|&(index, ref elements)| (index, elements.as_slice()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct BibliographyList<'t> {
    bibliographies: Vec<Bibliography<'t>>,
}

impl<'t> BibliographyList<'t> {
    pub fn new() -> Self {
        BibliographyList::default()
    }

    pub fn push(&mut self, bibliography: Bibliography<'t>) {
        self.bibliographies.push(bibliography);
    }

    pub fn get(&self, label: &str) -> Option<(u32, &[Element<'t>])> {
        for bibliography in &self.bibliographies {
            // Find the first entry with the label, per the above invariant.
            let reference = bibliography.get(label);
            if reference.is_some() {
                return reference;
            }
        }

        None
    }
}
