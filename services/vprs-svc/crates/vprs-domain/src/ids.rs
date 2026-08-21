use uuid::Uuid;

pub fn uuid7str() -> String {
    Uuid::now_v7().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid7str_produces_valid_uuid() {
        let id = uuid7str();
        assert!(!id.is_empty());
        assert_eq!(id.len(), 36);
    }
}
