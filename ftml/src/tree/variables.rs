/*
 * tree/variables.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};

pub type VariableMap<'t> = HashMap<Cow<'t, str>, Cow<'t, str>>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariableScopes<'t> {
    scopes: Vec<VariableMap<'t>>,
}

impl<'t> VariableScopes<'t> {
    #[inline]
    pub fn new() -> Self {
        VariableScopes::default()
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }

        None
    }

    pub fn push_scope(&mut self, scope: VariableMap<'t>) {
        self.scopes.push(scope);
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop().expect("Scope stack was empty");
    }
}

pub type VariableContextGetScopesFn<'t, C> = fn(&mut C) -> &mut VariableScopes<'t>;

/// A wrapper structure that automatically adds and removes a variable scope.
///
/// This permits render contexts to have confidence that they are in the proper scope,
/// even in the case of errors or other exceptional code paths.
pub struct VariableContextWrap<'t, C> {
    context: C,
    get_scopes_fn: VariableContextGetScopesFn<'t, C>,
}

impl<'t, C> VariableContextWrap<'t, C> {
    pub fn new(
        mut context: C,
        scope: VariableMap<'t>,
        get_scopes_fn: VariableContextGetScopesFn<'t, C>,
    ) -> Self {
        get_scopes_fn(&mut context).push_scope(scope);

        VariableContextWrap {
            context,
            get_scopes_fn,
        }
    }
}

impl<'t, C> Deref for VariableContextWrap<'t, C> {
    type Target = C;

    #[inline]
    fn deref(&self) -> &C {
        &self.context
    }
}

impl<'t, C> DerefMut for VariableContextWrap<'t, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut C {
        &mut self.context
    }
}

impl<'t, C> Drop for VariableContextWrap<'t, C> {
    fn drop(&mut self) {
        (self.get_scopes_fn)(&mut self.context).pop_scope();
    }
}

impl<'t, C> Debug for VariableContextWrap<'t, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VariableContextWrap")
            .field("context", &self.context)
            .field("get_scopes", &(self.get_scopes_fn as *const ()))
            .finish()
    }
}
