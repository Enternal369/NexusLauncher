use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Debug, Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionDetail {
    pub id: String,
    pub downloads: Downloads,
    pub libraries: Vec<Library>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    pub arguments: serde_json::Value,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: u64,
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    pub logging: Logging,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: u64,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Deserialize)]
pub struct Logging {
    pub client: Client,
}

#[derive(Debug, Deserialize)]
pub struct Client {
    pub argument: String,
    pub file: File,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Deserialize)]
pub struct File {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u64,
}

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub client: DownloadInfo,
    pub server: DownloadInfo,
}

#[derive(Debug, Deserialize)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Library {
    pub name: String,
    pub downloads: LibDownloads,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LibDownloads {
    pub artifact: Option<Artifact>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct AssetIndexManifest {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}
