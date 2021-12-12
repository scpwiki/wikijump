/*
 * render/html/random.rs
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

use cfg_if::cfg_if;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::iter;

#[cfg(test)]
const TEST_RANDOM_SEED: [u8; 32] = [
    0x53, 0x43, 0x50, 0x2d, 0x31, 0x37, 0x33, 0x3a, 0x20, 0x4d, 0x6f, 0x76, 0x65, 0x64,
    0x20, 0x74, 0x6f, 0x20, 0x53, 0x69, 0x74, 0x65, 0x2d, 0x31, 0x39, 0x20, 0x31, 0x39,
    0x39, 0x33, 0x2e, 0x0a,
];

#[derive(Debug)]
pub struct Random {
    rng: SmallRng,
}

impl Default for Random {
    #[inline]
    fn default() -> Self {
        cfg_if! {
            if #[cfg(test)] {
                let rng = SmallRng::from_seed(TEST_RANDOM_SEED);
            } else {
                let rng = SmallRng::from_entropy();
            }
        }

        Random { rng }
    }
}

impl Random {
    pub fn generate_html_id_into(&mut self, buffer: &mut String) {
        buffer.push_str("wj-id-");

        let char_stream = iter::repeat(())
            .map(|_| self.rng.sample(Alphanumeric))
            .map(char::from)
            .take(16);

        buffer.extend(char_stream);
    }

    pub fn generate_html_id(&mut self) -> String {
        let mut buffer = String::new();
        self.generate_html_id_into(&mut buffer);
        buffer
    }
}

#[test]
fn html_id() {
    // Random output is deterministic in tests.
    //
    // This is to ensure HTML test output is consistent,
    // but that means we can test for exact values here.

    let mut rand = Random::default();
    let mut buffer = String::new();

    rand.generate_html_id_into(&mut buffer);
    assert_eq!(
        buffer, "wj-id-bW5Ql2DLZtnd9s18",
        "Generated HTML ID doesn't match expected",
    );

    let html_id = rand.generate_html_id();
    assert_eq!(
        html_id, "wj-id-ePZbhugrfP89c4Fk",
        "Generated HTML ID doesn't match expected",
    );
}
