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

use crate::envelope::NcbCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrimeEntity {
    pub reference_number: String,
    pub crime_type: String,
    pub date_occurred: String,
    pub ncb_origin: NcbCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonEntity {
    pub family_name: String,
    pub given_name: String,
    pub birth_date: String,
    pub nationality: String,
    pub ncb_origin: NcbCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedEntity {
    pub crime: CrimeEntity,
    pub person: PersonEntity,
}

/// Associate a crime with a person of interest.
pub fn link_crime_to_person(crime: &CrimeEntity, person: &PersonEntity) -> LinkedEntity {
    LinkedEntity {
        crime: crime.clone(),
        person: person.clone(),
    }
}
