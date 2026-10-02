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

//! Compile-fail: calling reveal() without a Purpose must not compile.
//! This file is expected to fail with a type error.

use eel_pii_rs::{NoopAuditSink, PIIRestricted};

fn main() {
    let v = PIIRestricted::new("secret".to_string());
    // The following line should fail: reveal() requires a Purpose argument.
    let _ = v.reveal(&NoopAuditSink); //~ ERROR
}
