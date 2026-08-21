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
