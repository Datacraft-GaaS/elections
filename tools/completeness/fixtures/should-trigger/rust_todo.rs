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

// Fixture: should trigger todo_macro and unimplemented_macro findings.
fn not_ready() {
    todo!("implement me")
}

fn also_not_ready() {
    unimplemented!()
}

#[ignore]
#[test]
fn skipped_test() {
    assert_eq!(1, 1);
}

fn handler() -> Result<(), String> {
    // TODO: wire up the real handler
    Err("unimplemented".into())
}
