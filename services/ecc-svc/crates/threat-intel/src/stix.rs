use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum StixType {
    Indicator,
    ThreatActor,
    Campaign,
    Malware,
    AttackPattern,
    Tool,
    Vulnerability,
    IntrusionSet,
    Bundle,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonProps {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: StixType,
    pub spec_version: String,
    pub created: String,
    pub modified: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicator {
    #[serde(flatten)]
    pub common: CommonProps,
    pub pattern: Option<String>,
    pub pattern_type: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatActor {
    #[serde(flatten)]
    pub common: CommonProps,
    pub aliases: Option<Vec<String>>,
    pub threat_actor_types: Option<Vec<String>>,
    pub sophistication: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    #[serde(flatten)]
    pub common: CommonProps,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub objective: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Malware {
    #[serde(flatten)]
    pub common: CommonProps,
    pub malware_types: Option<Vec<String>>,
    pub is_family: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPattern {
    #[serde(flatten)]
    pub common: CommonProps,
    pub kill_chain_phases: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(flatten)]
    pub common: CommonProps,
    pub tool_types: Option<Vec<String>>,
    pub tool_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    #[serde(flatten)]
    pub common: CommonProps,
    pub external_references: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrusionSet {
    #[serde(flatten)]
    pub common: CommonProps,
    pub aliases: Option<Vec<String>>,
    pub goals: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StixObject {
    Indicator(Indicator),
    ThreatActor(ThreatActor),
    Campaign(Campaign),
    Malware(Malware),
    AttackPattern(AttackPattern),
    Tool(Tool),
    Vulnerability(Vulnerability),
    IntrusionSet(IntrusionSet),
    Unknown(serde_json::Value),
}

impl StixObject {
    pub fn type_str(&self) -> &str {
        match self {
            StixObject::Indicator(_) => "indicator",
            StixObject::ThreatActor(_) => "threat-actor",
            StixObject::Campaign(_) => "campaign",
            StixObject::Malware(_) => "malware",
            StixObject::AttackPattern(_) => "attack-pattern",
            StixObject::Tool(_) => "tool",
            StixObject::Vulnerability(_) => "vulnerability",
            StixObject::IntrusionSet(_) => "intrusion-set",
            StixObject::Unknown(v) => v.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"),
        }
    }

    pub fn id(&self) -> Option<&str> {
        match self {
            StixObject::Indicator(o) => Some(&o.common.id),
            StixObject::ThreatActor(o) => Some(&o.common.id),
            StixObject::Campaign(o) => Some(&o.common.id),
            StixObject::Malware(o) => Some(&o.common.id),
            StixObject::AttackPattern(o) => Some(&o.common.id),
            StixObject::Tool(o) => Some(&o.common.id),
            StixObject::Vulnerability(o) => Some(&o.common.id),
            StixObject::IntrusionSet(o) => Some(&o.common.id),
            StixObject::Unknown(v) => v.get("id").and_then(|i| i.as_str()),
        }
    }

    pub fn as_indicator(&self) -> Option<&Indicator> {
        match self {
            StixObject::Indicator(i) => Some(i),
            _ => None,
        }
    }
}

/// Parse a raw JSON string representing a STIX object, dispatching by "type" field.
pub fn parse_object_from_value(v: serde_json::Value) -> StixObject {
    let type_str = v
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_owned();

    match type_str.as_str() {
        "indicator" => serde_json::from_value::<Indicator>(v.clone())
            .map(StixObject::Indicator)
            .unwrap_or(StixObject::Unknown(v)),
        "threat-actor" => serde_json::from_value::<ThreatActor>(v.clone())
            .map(StixObject::ThreatActor)
            .unwrap_or(StixObject::Unknown(v)),
        "campaign" => serde_json::from_value::<Campaign>(v.clone())
            .map(StixObject::Campaign)
            .unwrap_or(StixObject::Unknown(v)),
        "malware" => serde_json::from_value::<Malware>(v.clone())
            .map(StixObject::Malware)
            .unwrap_or(StixObject::Unknown(v)),
        "attack-pattern" => serde_json::from_value::<AttackPattern>(v.clone())
            .map(StixObject::AttackPattern)
            .unwrap_or(StixObject::Unknown(v)),
        "tool" => serde_json::from_value::<Tool>(v.clone())
            .map(StixObject::Tool)
            .unwrap_or(StixObject::Unknown(v)),
        "vulnerability" => serde_json::from_value::<Vulnerability>(v.clone())
            .map(StixObject::Vulnerability)
            .unwrap_or(StixObject::Unknown(v)),
        "intrusion-set" => serde_json::from_value::<IntrusionSet>(v.clone())
            .map(StixObject::IntrusionSet)
            .unwrap_or(StixObject::Unknown(v)),
        _ => StixObject::Unknown(v),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StixBundle {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub objects: Vec<StixObject>,
}

pub fn parse_bundle(json: &str) -> Result<StixBundle, serde_json::Error> {
    // Parse as raw Value first, then dispatch each object by type
    let raw: serde_json::Value = serde_json::from_str(json)?;
    let id = raw
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let type_ = raw
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("bundle")
        .to_owned();
    let objects = raw
        .get("objects")
        .and_then(|o| o.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(parse_object_from_value)
        .collect();
    Ok(StixBundle { id, type_, objects })
}

pub fn parse_object(json: &str) -> Result<StixObject, serde_json::Error> {
    let v: serde_json::Value = serde_json::from_str(json)?;
    Ok(parse_object_from_value(v))
}
