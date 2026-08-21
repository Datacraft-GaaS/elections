//! Maps Kafka topics to tlog-domain Domain tags.

use tlog_domain::Domain;

/// Map a Kafka topic name to its domain tag.
pub fn topic_to_domain(topic: &str) -> Domain {
    if topic.starts_with("eel.results") || topic.starts_with("results") {
        Domain::ResultsEvents
    } else if topic.starts_with("eel.custody") || topic.starts_with("custody") {
        Domain::CustodyEvents
    } else if topic.starts_with("eel.credential") || topic.starts_with("credential") {
        Domain::CredentialEvents
    } else if topic.starts_with("eel.meta") || topic.starts_with("meta") {
        Domain::MetaEvents
    } else {
        Domain::VoterEvents
    }
}
