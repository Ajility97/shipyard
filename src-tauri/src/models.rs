use serde::{Deserialize, Serialize};

fn default_refresh_interval() -> u64 {
    300
}

fn default_files_pane_width() -> u32 {
    320
}

fn default_diff_mode() -> String {
    "split".into()
}

pub const MIN_WINDOW_WIDTH: u32 = 960;
pub const MIN_WINDOW_HEIGHT: u32 = 640;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub groups: Vec<RepoGroup>,
    #[serde(default)]
    pub repos: Vec<RepoEntry>,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_seconds: u64,
    #[serde(default = "default_files_pane_width")]
    pub files_pane_width: u32,
    #[serde(default = "default_diff_mode")]
    pub diff_mode: String,
    #[serde(default)]
    pub window: Option<WindowState>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            repos: Vec::new(),
            refresh_interval_seconds: default_refresh_interval(),
            files_pane_width: default_files_pane_width(),
            diff_mode: default_diff_mode(),
            window: None,
        }
    }
}

fn default_header_color() -> String {
    "#16323c".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoGroup {
    pub id: String,
    pub name: String,
    pub expanded: bool,
    pub pull_from_branch: String,
    pub checkout_fallbacks: Vec<String>,
    #[serde(default = "default_header_color")]
    pub header_color: String,
    pub repos: Vec<RepoEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_groups_as_camel_case_json() {
        let data = AppData {
            refresh_interval_seconds: 300,
            files_pane_width: 320,
            diff_mode: "split".into(),
            window: None,
            repos: Vec::new(),
            groups: vec![RepoGroup {
                id: "g1".into(),
                name: "Work".into(),
                expanded: true,
                pull_from_branch: "develop".into(),
                checkout_fallbacks: vec!["develop".into()],
                header_color: "#16323c".into(),
                repos: vec![RepoEntry {
                    id: "r1".into(),
                    path: "/tmp/api".into(),
                    label: String::new(),
                    header_color: String::new(),
                }],
            }],
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"pullFromBranch\":\"develop\""));
        assert!(json.contains("\"checkoutFallbacks\""));
        let parsed: AppData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.groups[0].repos[0].path, "/tmp/api");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoEntry {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub header_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatus {
    pub id: String,
    pub path: String,
    pub name: String,
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub dirty: bool,
    pub insertions: u32,
    pub deletions: u32,
    pub changed_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoActionResult {
    pub path: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitNode {
    pub hash: String,
    pub parents: Vec<String>,
    pub subject: String,
    pub author: String,
    pub date: String,
    pub refs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingTreeFile {
    pub path: String,
    pub status: String,
    pub untracked: bool,
    #[serde(default)]
    pub staged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitFile {
    pub path: String,
    pub old_path: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalBranch {
    pub name: String,
    pub current: bool,
    pub merged: bool,
    pub partial: bool,
    #[serde(rename = "protected")]
    pub protected_branch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMergedResult {
    pub deleted: Vec<String>,
    pub refused: Vec<String>,
    pub errors: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchOverview {
    pub merge_target: Option<String>,
    pub branches: Vec<LocalBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StashEntry {
    pub index: u32,
    pub message: String,
    pub date: String,
}
