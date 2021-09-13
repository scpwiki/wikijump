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
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub type VariableMap<'t> = HashMap<Cow<'t, str>, Cow<'t, str>>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariableScopes<'v, 't> {
    scopes: Vec<&'v VariableMap<'t>>,
}

impl<'v, 't> VariableScopes<'v, 't> {
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

    pub fn push_scope(&mut self, scope: &'v VariableMap<'t>) {
        self.scopes.push(scope);
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop().expect("Scope stack was empty");
    }
}

pub trait VariableContext<'v, 't> {
    fn get_scopes(&mut self) -> &mut VariableScopes<'v, 't>;
}

/// A wrapper structure that automatically adds and removes a variable scope.
///
/// This permits render contexts to have confidence that they are in the proper scope,
/// even in the case of errors or other exceptional code paths.
#[derive(Debug)]
pub struct VariableContextWrap<'c, 'v, 't, C>
where
    C: VariableContext<'v, 't>,
{
    context: &'c mut C,
    _marker: PhantomData<&'v VariableMap<'t>>,
}

impl<'c, 'v, 't, C> VariableContextWrap<'c, 'v, 't, C>
where
    C: VariableContext<'v, 't>,
{
    pub fn new(context: &'c mut C, scope: &'v VariableMap<'t>) -> Self {
        context.get_scopes().push_scope(scope);

        VariableContextWrap {
            context,
            _marker: PhantomData,
        }
    }
}

impl<'c, 'v, 't, C> Deref for VariableContextWrap<'c, 'v, 't, C>
where
    C: VariableContext<'v, 't>,
{
    type Target = C;

    #[inline]
    fn deref(&self) -> &C {
        self.context
    }
}

impl<'c, 'v, 't, C> DerefMut for VariableContextWrap<'c, 'v, 't, C>
where
    C: VariableContext<'v, 't>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut C {
        self.context
    }
}

impl<'c, 'v, 't, C> Drop for VariableContextWrap<'c, 'v, 't, C>
where
    C: VariableContext<'v, 't>,
{
    fn drop(&mut self) {
        self.context.get_scopes().pop_scope();
    }
}
