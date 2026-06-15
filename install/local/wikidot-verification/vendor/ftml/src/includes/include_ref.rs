/*
 * includes/include_ref.rs
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

use crate::data::PageRef;
use crate::tree::VariableMap;

/// Represents an include block before it has been replaced with the fetched page.
///
/// It contains the page being included, as well as the variables passed to it in the include
/// block.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct IncludeRef<'t> {
    page_ref: PageRef,
    variables: VariableMap<'t>,
}

impl<'t> IncludeRef<'t> {
    #[inline]
    pub fn new(page_ref: PageRef, variables: VariableMap<'t>) -> Self {
        IncludeRef {
            page_ref,
            variables,
        }
    }

    #[inline]
    pub fn page_only(page_ref: PageRef) -> Self {
        IncludeRef::new(page_ref, VariableMap::new())
    }

    #[inline]
    pub fn page_ref(&self) -> &PageRef {
        &self.page_ref
    }

    #[inline]
    pub fn variables(&self) -> &VariableMap<'t> {
        &self.variables
    }
}

impl<'t> From<IncludeRef<'t>> for (PageRef, VariableMap<'t>) {
    #[inline]
    fn from(include: IncludeRef<'t>) -> (PageRef, VariableMap<'t>) {
        let IncludeRef {
            page_ref,
            variables,
        } = include;

        (page_ref, variables)
    }
}

// Tests

#[test]
fn to_owned() {
    // Clone PageRef
    let page_ref_1 = PageRef::page_only("scp-001");
    let page_ref_2 = page_ref_1.clone();
    assert_eq!(page_ref_1, page_ref_2);

    // Clone IncludeRef
    let include_ref_1 = IncludeRef::new(page_ref_1, VariableMap::new());
    let include_ref_2: IncludeRef<'static> = include_ref_1.to_owned();
    assert_eq!(include_ref_1, include_ref_2);
    assert_eq!(include_ref_1.page_ref(), &page_ref_2);
    assert!(include_ref_1.variables.is_empty());

    // Deconstruct IncludeRef
    let (page_ref, variables) = include_ref_2.into();
    assert_eq!(page_ref, page_ref_2);
    assert!(variables.is_empty());
}
