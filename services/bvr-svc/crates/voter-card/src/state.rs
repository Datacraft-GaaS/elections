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

use sqlx::PgPool;

use crate::store::Store;

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self { store: Store::pg(pool.clone()) }
    }
    pub fn new_mem() -> Self {
        Self { store: Store::mem() }
    }
    /// Pool accessor for legacy routes that do direct sqlx (batches, blacklist, status).
    /// Panics in `Mem` mode — those routes are not used in hermetic tests.
    pub fn pool(&self) -> &PgPool {
        match &self.store {
            Store::Pg(p) => p,
            Store::Mem(_) => panic!("pool not available in Mem store — use Store::Pg in prod"),
        }
    }
}
