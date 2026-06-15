/*
 * render/html/random.rs
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

use cfg_if::cfg_if;
use rand::distr::{Alphanumeric, SampleString};
use rand::rngs::SmallRng;

#[derive(Debug)]
pub struct Random {
    rng: SmallRng,
}

impl Default for Random {
    #[inline]
    fn default() -> Self {
        cfg_if! {
            if #[cfg(test)] {
                use rand::SeedableRng;
                let rng = SmallRng::seed_from_u64(1);
            } else {
                let rng = rand::make_rng();
            }
        }

        Random { rng }
    }
}

impl Random {
    pub fn generate_html_id_into(&mut self, buffer: &mut String) {
        buffer.push_str("wj-id-");
        Alphanumeric.append_string(&mut self.rng, buffer, 16);
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
        buffer, "wj-id-zvGvLlhGI6VEZFKj",
        "Generated HTML ID doesn't match expected",
    );

    let html_id = rand.generate_html_id();
    assert_eq!(
        html_id, "wj-id-e9pQyKaPmLnulpgn",
        "Generated HTML ID doesn't match expected",
    );
}
