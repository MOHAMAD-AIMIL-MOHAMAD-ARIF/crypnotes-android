pub const DOC_SCHEMA_VERSION: u32 = 1;
pub const NOTE_PAYLOAD_SCHEMA_VERSION: u32 = 1;
pub const ENCRYPTION_CONTAINER_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionSet {
    pub doc_schema_version: u32,
    pub note_payload_schema_version: u32,
    pub encryption_container_version: u32,
}

pub const CURRENT_VERSIONS: VersionSet = VersionSet {
    doc_schema_version: DOC_SCHEMA_VERSION,
    note_payload_schema_version: NOTE_PAYLOAD_SCHEMA_VERSION,
    encryption_container_version: ENCRYPTION_CONTAINER_VERSION,
};

pub fn current_versions() -> VersionSet {
    CURRENT_VERSIONS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_versions_match_individual_constants() {
        let versions = current_versions();
        assert_eq!(versions.doc_schema_version, DOC_SCHEMA_VERSION);
        assert_eq!(
            versions.note_payload_schema_version,
            NOTE_PAYLOAD_SCHEMA_VERSION
        );
        assert_eq!(
            versions.encryption_container_version,
            ENCRYPTION_CONTAINER_VERSION
        );
    }
}
