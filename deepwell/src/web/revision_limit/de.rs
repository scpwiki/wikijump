/*
 * web/revision_limit/de.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

use super::RevisionLimit;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use std::fmt;

impl<'de> Deserialize<'de> for RevisionLimit {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u16(RevisionLimitVisitor)
    }
}

#[derive(Debug)]
struct RevisionLimitVisitor;

impl RevisionLimitVisitor {
    fn visit_unsigned<E>(&self, value: u128) -> Result<RevisionLimit, E>
    where
        E: de::Error,
    {
        if value <= 100 {
            Ok(RevisionLimit(value as u16))
        } else {
            Err(E::custom(format!("limit out of range: {}", value)))
        }
    }

    fn visit_signed<E>(&self, value: i128) -> Result<RevisionLimit, E>
    where
        E: de::Error,
    {
        if value >= 0 {
            self.visit_unsigned(value as u128)
        } else {
            Err(E::custom(format!("limit must be positive: {}", value)))
        }
    }
}

macro_rules! impl_visit_unsigned {
    ($type:ty, $impl_method:ident) => {
        #[inline]
        fn $impl_method<E>(self, value: $type) -> Result<RevisionLimit, E>
        where
            E: de::Error,
        {
            self.visit_unsigned(u128::from(value))
        }
    };
}

macro_rules! impl_visit_signed {
    ($type:ty, $impl_method:ident) => {
        #[inline]
        fn $impl_method<E>(self, value: $type) -> Result<RevisionLimit, E>
        where
            E: de::Error,
        {
            self.visit_signed(i128::from(value))
        }
    };
}

impl<'de> Visitor<'de> for RevisionLimitVisitor {
    type Value = RevisionLimit;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between 0 and 100")
    }

    impl_visit_signed!(i8, visit_i8);
    impl_visit_signed!(i16, visit_i16);
    impl_visit_signed!(i32, visit_i32);
    impl_visit_signed!(i64, visit_i64);
    impl_visit_signed!(i128, visit_i128);

    impl_visit_unsigned!(u8, visit_u8);
    impl_visit_unsigned!(u16, visit_u16);
    impl_visit_unsigned!(u32, visit_u32);
    impl_visit_unsigned!(u64, visit_u64);
    impl_visit_unsigned!(u128, visit_u128);
}
