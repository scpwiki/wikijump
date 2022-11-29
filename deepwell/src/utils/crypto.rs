/*
 * utils/crypto.rs
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

//! Utilities related to cryptographic operations and primitives.

use rand::CryptoRng;

/// Statically verifies that this random number generator is secure.
///
/// The build will fail if the passed generator is not a CSPRNG
/// (cryptographically-secure psuedorandom number generator).
#[inline]
pub fn assert_is_csprng(_: &dyn CryptoRng) {}
