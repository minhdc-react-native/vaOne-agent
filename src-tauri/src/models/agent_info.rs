use serde::Serialize;

#[derive(Serialize)]
pub struct AgentInfo {
    pub name: String,
    pub version: String,
    pub os: String,
}
