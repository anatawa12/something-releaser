use crate::release_system::VersionName;

pub struct VersionInfo {
    pub version: VersionName,
    pub release_note_html: String,
    pub release_note_markdown: String,
}
