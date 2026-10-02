// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Name generation from bundled fixture files.
// All names are synthetic. None are derived from real individuals.

use rand::Rng;
use rand_chacha::ChaCha20Rng;

static FIRST_NAMES_RAW: &str = include_str!("../fixtures/first_names.txt");
static LAST_NAMES_RAW: &str = include_str!("../fixtures/last_names.txt");

pub struct NamePool {
    first: Vec<&'static str>,
    last: Vec<&'static str>,
}

impl NamePool {
    pub fn load() -> Self {
        let first: Vec<&'static str> = FIRST_NAMES_RAW
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let last: Vec<&'static str> = LAST_NAMES_RAW
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        assert!(!first.is_empty(), "first_names.txt fixture is empty");
        assert!(!last.is_empty(), "last_names.txt fixture is empty");
        Self { first, last }
    }

    pub fn first(&self, rng: &mut ChaCha20Rng) -> &'static str {
        self.first[rng.gen_range(0..self.first.len())]
    }

    pub fn last(&self, rng: &mut ChaCha20Rng) -> &'static str {
        self.last[rng.gen_range(0..self.last.len())]
    }

    pub fn full(&self, rng: &mut ChaCha20Rng) -> String {
        format!("{} {}", self.first(rng), self.last(rng))
    }

    pub fn first_count(&self) -> usize {
        self.first.len()
    }

    pub fn last_count(&self) -> usize {
        self.last.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::seeded;

    #[test]
    fn fixture_loads() {
        let pool = NamePool::load();
        assert!(pool.first_count() >= 50);
        assert!(pool.last_count() >= 50);
    }

    #[test]
    fn name_generation_is_deterministic() {
        let pool = NamePool::load();
        let mut rng_a = seeded(99);
        let mut rng_b = seeded(99);
        let a = pool.full(&mut rng_a);
        let b = pool.full(&mut rng_b);
        assert_eq!(a, b);
    }
}
