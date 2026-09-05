    pub source_ref: Option<Id>,
    pub provenance_ref: Option<Id>,
    pub integrity_status: IntegrityStatus,
    pub supersedes_version_id: Option<Id>,
    pub language: Option<String>,
    pub created_at: String,
}

impl DocumentVersion {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.document_version_id.is_empty() || self.document_id.is_empty() {
            return Err("document version identity is required");
        }
        if self.schema_version == 0 || self.version_number == 0 {
            return Err("document version number and schema version are required");
        }
        if self.media_type.is_empty() || self.content_ref.is_empty() || self.content_hash.is_empty() {
            return Err("document version content metadata is required");
        }
        if self.created_by.is_empty() || self.created_at.is_empty() {
            return Err("document version creator and timestamp are required");
        }
        Ok(())