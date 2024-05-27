use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum BuildState {
    #[serde(rename = "failing")]
    Failing,
    #[serde(rename = "schedule")]
    Scheduled,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "canceled")]
    Canceled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub web_url: String,
    pub repository: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
    pub number: String,
    pub commit: String,
    pub created_at: String,
    pub state: BuildState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildData {
    pub pipeline: Pipeline,
    pub build: Build,
    pub sender_name: String,
    pub creator_avatar: String,
}

// impl BuildData {
//     pub fn pipeline(self) -> Pipeline { self.pipeline }
// }
