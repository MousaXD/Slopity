use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ServerId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeKind {
    Java,
    NodeJs,
    Python,
    Php,
    Native,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkScope {
    Loopback,
    Lan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerProfile {
    pub id: ServerId,
    pub name: String,
    pub runtime: RuntimeKind,
    pub executable: Option<PathBuf>,
    pub arguments: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub port: u16,
    pub memory_mib: u32,
    pub network_scope: NetworkScope,
    pub enabled: bool,
}

pub fn sample_profiles() -> Vec<ServerProfile> {
    vec![
        ServerProfile {
            id: ServerId("paper-example".into()),
            name: "Paper example".into(),
            runtime: RuntimeKind::Java,
            executable: None,
            arguments: vec!["-jar".into(), "paper.jar".into(), "--nogui".into()],
            working_directory: None,
            port: 25_565,
            memory_mib: 2_048,
            network_scope: NetworkScope::Loopback,
            enabled: false,
        },
        ServerProfile {
            id: ServerId("node-example".into()),
            name: "Node.js example".into(),
            runtime: RuntimeKind::NodeJs,
            executable: None,
            arguments: vec!["server.js".into()],
            working_directory: None,
            port: 3_000,
            memory_mib: 512,
            network_scope: NetworkScope::Loopback,
            enabled: false,
        },
        ServerProfile {
            id: ServerId("native-example".into()),
            name: "Native example".into(),
            runtime: RuntimeKind::Native,
            executable: None,
            arguments: Vec::new(),
            working_directory: None,
            port: 8_080,
            memory_mib: 256,
            network_scope: NetworkScope::Loopback,
            enabled: false,
        },
    ]
}
