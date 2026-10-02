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

use std::collections::HashMap;

/// Registry of known CBEFF format owners (IBIA registry).
pub struct FormatRegistry {
    known_format_owners: HashMap<u16, &'static str>,
}

impl FormatRegistry {
    /// Initialise with the subset of well-known format owners.
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert(0x0001, "ISO/IEC JTC1 SC37");
        m.insert(0x0101, "FBI");
        m.insert(0x0201, "NIST");
        m.insert(0x0301, "Interpol");
        Self {
            known_format_owners: m,
        }
    }

    /// Look up the name of a format owner by its registered ID.
    pub fn lookup_owner(&self, id: u16) -> Option<&str> {
        self.known_format_owners.get(&id).copied()
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}
