use serde::{Deserialize, Serialize};

fn default_refresh_interval() -> u64 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub groups: Vec<RepoGroup>,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_seconds: u64,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            refresh_interval_seconds: default_refresh_interval(),
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
}
