use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum IdError {
    #[error("invalid uuid: {0}")]
    InvalidUuid(String),
}

pub fn new_uuid_v7() -> Uuid {
    Uuid::now_v7()
}

pub fn parse_uuid(input: &str) -> Result<Uuid, IdError> {
    Uuid::parse_str(input).map_err(|_| IdError::InvalidUuid(input.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_v7_is_generated() {
        let id = new_uuid_v7();
        assert_eq!(id.get_version_num(), 7);
    }

    #[test]
    fn parse_invalid_uuid_fails() {
        let result = parse_uuid("not-a-uuid");
        assert!(matches!(result, Err(IdError::InvalidUuid(_))));
    }
}
