//! Integration tests for the transparency log service.

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use tlog_domain::{Domain, LeafFormat};

    #[test]
    fn leaf_hash_smoke() {
        let t = Utc::now();
        let h = LeafFormat::hash(b"test event", "spiffe://eel.local/test", t, 0);
        assert_eq!(h.len(), 32);
    }

    #[test]
    fn domain_all_variants_parse() {
        use std::str::FromStr;
        for d in [
            "voter-events",
            "results-events",
            "custody-events",
            "credential-events",
            "meta-events",
        ] {
            assert!(Domain::from_str(d).is_ok(), "failed to parse domain: {}", d);
        }
    }
}
