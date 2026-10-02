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

//! Kenya's 47 county code → name mapping (KE-ISO 3166-2 codes `001`..`047`).

/// Return the county name for a three-digit county code, or the code itself
/// when unknown (should not happen for validated `001`..`047`).
pub fn county_name(code: &str) -> &str {
    match code {
        "001" => "Mombasa",
        "002" => "Kwale",
        "003" => "Kilifi",
        "004" => "Tana River",
        "005" => "Lamu",
        "006" => "Taita-Taveta",
        "007" => "Garissa",
        "008" => "Wajir",
        "009" => "Mandera",
        "010" => "Marsabit",
        "011" => "Isiolo",
        "012" => "Meru",
        "013" => "Tharaka-Nithi",
        "014" => "Embu",
        "015" => "Kitui",
        "016" => "Machakos",
        "017" => "Makueni",
        "018" => "Nyandarua",
        "019" => "Nyeri",
        "020" => "Kirinyaga",
        "021" => "Murang'a",
        "022" => "Kiambu",
        "023" => "Turkana",
        "024" => "West Pokot",
        "025" => "Samburu",
        "026" => "Trans Nzoia",
        "027" => "Uasin Gishu",
        "028" => "Elgeyo-Marakwet",
        "029" => "Nandi",
        "030" => "Baringo",
        "031" => "Laikipia",
        "032" => "Nakuru",
        "033" => "Narok",
        "034" => "Kajiado",
        "035" => "Kericho",
        "036" => "Bomet",
        "037" => "Kakamega",
        "038" => "Vihiga",
        "039" => "Bungoma",
        "040" => "Busia",
        "041" => "Siaya",
        "042" => "Kisumu",
        "043" => "Homa Bay",
        "044" => "Migori",
        "045" => "Kisii",
        "046" => "Nyamira",
        "047" => "Nairobi",
        _ => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_counties_map_to_names() {
        assert_eq!(county_name("001"), "Mombasa");
        assert_eq!(county_name("047"), "Nairobi");
        assert_eq!(county_name("032"), "Nakuru");
    }

    #[test]
    fn unknown_code_falls_back() {
        assert_eq!(county_name("999"), "999");
    }
}
