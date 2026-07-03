/*
 * utils/int.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

//! Fallible conversion helpers between integer types.
//!
//! These functions exist because, with `.or_raise()`,
//! Rust cannot determine the type we want to convert
//! to unless we're explicit.

use crate::error::StdResult;
use std::num::TryFromIntError;

macro_rules! impl_convert_traits {
    ($int:ty => i16) => {
        impl ConvertToI16 for $int {
            fn try_into_i16(self) -> StdResult<i16, TryFromIntError> {
                self.try_into()
            }
        }
    };

    ($int:ty => i32) => {
        impl ConvertToI32 for $int {
            fn try_into_i32(self) -> StdResult<i32, TryFromIntError> {
                self.try_into()
            }
        }
    };

    ($int:ty => i64) => {
        impl ConvertToI64 for $int {
            fn try_into_i64(self) -> StdResult<i64, TryFromIntError> {
                self.try_into()
            }
        }
    };

    ($int:ty => u64) => {
        impl ConvertToU64 for $int {
            fn try_into_u64(self) -> StdResult<u64, TryFromIntError> {
                self.try_into()
            }
        }
    };

    ($int:ty => usize) => {
        impl ConvertToUsize for $int {
            fn try_into_usize(self) -> StdResult<usize, TryFromIntError> {
                self.try_into()
            }
        }
    };
}

pub trait ConvertToI16 {
    fn try_into_i16(self) -> StdResult<i16, TryFromIntError>;
}

pub trait ConvertToI32 {
    fn try_into_i32(self) -> StdResult<i32, TryFromIntError>;
}

pub trait ConvertToI64 {
    fn try_into_i64(self) -> StdResult<i64, TryFromIntError>;
}

pub trait ConvertToU64 {
    fn try_into_u64(self) -> StdResult<u64, TryFromIntError>;
}

pub trait ConvertToUsize {
    fn try_into_usize(self) -> StdResult<usize, TryFromIntError>;
}

impl_convert_traits!(i16 => usize);

impl_convert_traits!(i64 => i16);
impl_convert_traits!(i64 => i32);
impl_convert_traits!(i64 => u64);
impl_convert_traits!(i64 => usize);

impl_convert_traits!(u64 => i16);
impl_convert_traits!(u64 => i32);
impl_convert_traits!(u64 => i64);
impl_convert_traits!(u64 => usize);

impl_convert_traits!(usize => i16);
impl_convert_traits!(usize => i32);
impl_convert_traits!(usize => i64);
impl_convert_traits!(usize => u64);

#[test]
fn integer_conversions_accept_representable_values() {
    assert_eq!(9_i16.try_into_usize().unwrap(), 9_usize);

    assert_eq!(9_i64.try_into_i16().unwrap(), 9_i16);
    assert_eq!(9_i64.try_into_i32().unwrap(), 9_i32);
    assert_eq!(9_i64.try_into_u64().unwrap(), 9_u64);
    assert_eq!(9_i64.try_into_usize().unwrap(), 9_usize);

    assert_eq!(9_u64.try_into_i16().unwrap(), 9_i16);
    assert_eq!(9_u64.try_into_i32().unwrap(), 9_i32);
    assert_eq!(9_u64.try_into_i64().unwrap(), 9_i64);
    assert_eq!(9_u64.try_into_usize().unwrap(), 9_usize);

    assert_eq!(9_usize.try_into_i16().unwrap(), 9_i16);
    assert_eq!(9_usize.try_into_i32().unwrap(), 9_i32);
    assert_eq!(9_usize.try_into_i64().unwrap(), 9_i64);
    assert_eq!(9_usize.try_into_u64().unwrap(), 9_u64);
}

#[test]
fn integer_conversions_reject_out_of_range_values() {
    assert!(i16::MIN.try_into_usize().is_err());

    assert!((i16::MAX as i64 + 1).try_into_i16().is_err());
    assert!((i32::MAX as i64 + 1).try_into_i32().is_err());
    assert!((-1_i64).try_into_u64().is_err());
    assert!((-1_i64).try_into_usize().is_err());

    assert!((i16::MAX as u64 + 1).try_into_i16().is_err());
    assert!((i32::MAX as u64 + 1).try_into_i32().is_err());
    assert!((i64::MAX as u64 + 1).try_into_i64().is_err());

    assert!((i16::MAX as usize + 1).try_into_i16().is_err());
    assert!((i32::MAX as usize + 1).try_into_i32().is_err());

    #[cfg(target_pointer_width = "64")]
    assert!((i64::MAX as usize + 1).try_into_i64().is_err());
}
