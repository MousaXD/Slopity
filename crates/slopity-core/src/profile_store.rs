use crate::{validate_profile, ServerId, ServerProfile, ValidationSeverity};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;

pub const PROFILE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDocument {
    pub schema_version: u32,
    pub profiles: Vec<ServerProfile>,
}

impl ProfileDocument {
    pub fn new(profiles: Vec<ServerProfile>) -> Result<Self, ProfileStoreError> {
        validate_collection(&profiles)?;
        Ok(Self {
            schema_version: PROFILE_SCHEMA_VERSION,
            profiles,
        })
    }
}

#[derive(Debug, Error)]
pub enum ProfileStoreError {
    #[error("profile storage I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("profile document JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported profile schema version {found}; this build supports {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("profile ID already exists: {0}")]
    DuplicateId(String),
    #[error("profile not found: {0}")]
    NotFound(String),
    #[error("profile validation failed: {0}")]
    Validation(String),
    #[error("port {port} is assigned to both {first} and {second}")]
    PortConflict {
        port: u16,
        first: String,
        second: String,
    },
    #[error(
        "profile storage replacement failed ({write_error}); rollback also failed ({rollback_error})"
    )]
    Recovery {
        write_error: String,
        rollback_error: String,
    },
}

#[derive(Debug)]
pub struct ProfileStore {
    path: PathBuf,
    document: ProfileDocument,
}

impl ProfileStore {
    pub fn load_or_create(
        path: impl Into<PathBuf>,
        seed_profiles: Vec<ServerProfile>,
    ) -> Result<Self, ProfileStoreError> {
        let path = path.into();
        if path.exists() {
            let document: ProfileDocument = serde_json::from_slice(&fs::read(&path)?)?;
            validate_document(&document)?;
            return Ok(Self { path, document });
        }

        let document = ProfileDocument::new(seed_profiles)?;
        write_document(&path, &document)?;
        Ok(Self { path, document })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn profiles(&self) -> &[ServerProfile] {
        &self.document.profiles
    }

    pub fn profile(&self, id: &ServerId) -> Option<&ServerProfile> {
        self.document
            .profiles
            .iter()
            .find(|profile| &profile.id == id)
    }

    pub fn validation_issues(&self, profile: &ServerProfile) -> Vec<crate::ValidationIssue> {
        let reserved_ports = self
            .document
            .profiles
            .iter()
            .filter(|existing| existing.id != profile.id)
            .map(|existing| existing.port)
            .collect();
        validate_profile(profile, &reserved_ports)
    }

    pub fn create(&mut self, profile: ServerProfile) -> Result<ServerProfile, ProfileStoreError> {
        if self.profile(&profile.id).is_some() {
            return Err(ProfileStoreError::DuplicateId(profile.id.0));
        }

        reject_profile_errors(&profile, &self.validation_issues(&profile))?;
        let created = profile.clone();
        let mut profiles = self.document.profiles.clone();
        profiles.push(profile);
        self.commit_profiles(profiles)?;
        Ok(created)
    }

    pub fn update(&mut self, profile: ServerProfile) -> Result<ServerProfile, ProfileStoreError> {
        let index = self
            .document
            .profiles
            .iter()
            .position(|existing| existing.id == profile.id)
            .ok_or_else(|| ProfileStoreError::NotFound(profile.id.0.clone()))?;

        reject_profile_errors(&profile, &self.validation_issues(&profile))?;
        let updated = profile.clone();
        let mut profiles = self.document.profiles.clone();
        profiles[index] = profile;
        self.commit_profiles(profiles)?;
        Ok(updated)
    }

    pub fn clone_profile(
        &mut self,
        source_id: &ServerId,
        new_id: ServerId,
        new_name: String,
    ) -> Result<ServerProfile, ProfileStoreError> {
        let mut cloned = self
            .profile(source_id)
            .cloned()
            .ok_or_else(|| ProfileStoreError::NotFound(source_id.0.clone()))?;
        cloned.id = new_id;
        cloned.name = new_name;
        cloned.enabled = false;
        self.create(cloned)
    }

    pub fn set_enabled(
        &mut self,
        id: &ServerId,
        enabled: bool,
    ) -> Result<ServerProfile, ProfileStoreError> {
        let mut profile = self
            .profile(id)
            .cloned()
            .ok_or_else(|| ProfileStoreError::NotFound(id.0.clone()))?;
        profile.enabled = enabled;
        self.update(profile)
    }

    pub fn delete(&mut self, id: &ServerId) -> Result<ServerProfile, ProfileStoreError> {
        let index = self
            .document
            .profiles
            .iter()
            .position(|profile| &profile.id == id)
            .ok_or_else(|| ProfileStoreError::NotFound(id.0.clone()))?;

        let mut profiles = self.document.profiles.clone();
        let deleted = profiles.remove(index);
        self.commit_profiles(profiles)?;
        Ok(deleted)
    }

    fn commit_profiles(&mut self, profiles: Vec<ServerProfile>) -> Result<(), ProfileStoreError> {
        let document = ProfileDocument::new(profiles)?;
        write_document(&self.path, &document)?;
        self.document = document;
        Ok(())
    }
}

fn validate_document(document: &ProfileDocument) -> Result<(), ProfileStoreError> {
    if document.schema_version != PROFILE_SCHEMA_VERSION {
        return Err(ProfileStoreError::UnsupportedSchema {
            found: document.schema_version,
            supported: PROFILE_SCHEMA_VERSION,
        });
    }
    validate_collection(&document.profiles)
}

fn validate_collection(profiles: &[ServerProfile]) -> Result<(), ProfileStoreError> {
    let mut ids = HashSet::new();
    let mut ports = HashMap::new();

    for profile in profiles {
        if !ids.insert(profile.id.0.clone()) {
            return Err(ProfileStoreError::DuplicateId(profile.id.0.clone()));
        }
        if let Some(first) = ports.insert(profile.port, profile.id.0.clone()) {
            return Err(ProfileStoreError::PortConflict {
                port: profile.port,
                first,
                second: profile.id.0.clone(),
            });
        }
        reject_profile_errors(profile, &validate_profile(profile, &HashSet::new()))?;
    }

    Ok(())
}

fn reject_profile_errors(
    profile: &ServerProfile,
    issues: &[crate::ValidationIssue],
) -> Result<(), ProfileStoreError> {
    let errors = issues
        .iter()
        .filter(|issue| issue.severity == ValidationSeverity::Error)
        .map(|issue| format!("{}: {}", issue.code, issue.message))
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ProfileStoreError::Validation(format!(
            "{} ({})",
            profile.id.0,
            errors.join("; ")
        )))
    }
}

fn write_document(path: &Path, document: &ProfileDocument) -> Result<(), ProfileStoreError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let mut bytes = serde_json::to_vec_pretty(document)?;
    bytes.push(b'\n');

    let temporary_path = path.with_extension("tmp");
    let backup_path = path.with_extension("bak");
    if temporary_path.exists() {
        fs::remove_file(&temporary_path)?;
    }

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary_path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);

    match fs::rename(&temporary_path, path) {
        Ok(()) => Ok(()),
        Err(_) if path.exists() => {
            if backup_path.exists() {
                fs::remove_file(&backup_path)?;
            }
            fs::rename(path, &backup_path)?;
            match fs::rename(&temporary_path, path) {
                Ok(()) => {
                    let _ = fs::remove_file(backup_path);
                    Ok(())
                }
                Err(write_error) => match fs::rename(&backup_path, path) {
                    Ok(()) => Err(ProfileStoreError::Io(write_error)),
                    Err(rollback_error) => Err(ProfileStoreError::Recovery {
                        write_error: write_error.to_string(),
                        rollback_error: rollback_error.to_string(),
                    }),
                },
            }
        }
        Err(error) => Err(ProfileStoreError::Io(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NetworkScope, RuntimeKind};
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    fn test_directory() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "slopity-profile-store-{}-{}",
            std::process::id(),
            NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("test directory should be created");
        path
    }

    fn profile(id: &str, port: u16) -> ServerProfile {
        ServerProfile {
            id: ServerId(id.into()),
            name: id.into(),
            runtime: RuntimeKind::Native,
            executable: Some(PathBuf::from("runtime")),
            arguments: Vec::new(),
            working_directory: None,
            port,
            memory_mib: 512,
            network_scope: NetworkScope::Loopback,
            enabled: false,
        }
    }

    #[test]
    fn missing_store_creates_schema_v1_document() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        let store = ProfileStore::load_or_create(&path, vec![profile("alpha", 3_000)])
            .expect("store should be created");

        assert_eq!(store.profiles().len(), 1);
        let document: ProfileDocument =
            serde_json::from_slice(&fs::read(&path).expect("document should exist"))
                .expect("document should parse");
        assert_eq!(document.schema_version, PROFILE_SCHEMA_VERSION);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn malformed_and_future_documents_are_rejected() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        fs::write(&path, b"not-json").expect("fixture should be written");
        assert!(matches!(
            ProfileStore::load_or_create(&path, Vec::new()),
            Err(ProfileStoreError::Json(_))
        ));

        fs::write(&path, br#"{"schemaVersion":99,"profiles":[]}"#)
            .expect("future fixture should be written");
        assert!(matches!(
            ProfileStore::load_or_create(&path, Vec::new()),
            Err(ProfileStoreError::UnsupportedSchema { found: 99, .. })
        ));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn crud_operations_survive_reload() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        let mut store =
            ProfileStore::load_or_create(&path, Vec::new()).expect("store should be created");

        store
            .create(profile("alpha", 3_000))
            .expect("profile should be created");
        let mut updated = profile("alpha", 3_001);
        updated.name = "Updated alpha".into();
        store.update(updated).expect("profile should be updated");
        store
            .clone_profile(
                &ServerId("alpha".into()),
                ServerId("beta".into()),
                "Beta clone".into(),
            )
            .expect("profile should be cloned");
        store
            .set_enabled(&ServerId("beta".into()), true)
            .expect("clone should be enabled");
        store
            .delete(&ServerId("alpha".into()))
            .expect("source should be deleted");
        drop(store);

        let reopened =
            ProfileStore::load_or_create(&path, Vec::new()).expect("store should reload");
        assert_eq!(reopened.profiles().len(), 1);
        assert_eq!(reopened.profiles()[0].id.0, "beta");
        assert!(reopened.profiles()[0].enabled);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn duplicate_ports_do_not_replace_in_memory_state() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        let mut store = ProfileStore::load_or_create(&path, vec![profile("alpha", 3_000)])
            .expect("store should be created");
        let before = store.profiles().to_vec();

        assert!(matches!(
            store.create(profile("beta", 3_000)),
            Err(ProfileStoreError::Validation(_)) | Err(ProfileStoreError::PortConflict { .. })
        ));
        assert_eq!(store.profiles(), before.as_slice());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn failed_write_does_not_replace_in_memory_state() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        let mut store = ProfileStore::load_or_create(&path, vec![profile("alpha", 3_000)])
            .expect("store should be created");
        let before = store.profiles().to_vec();
        let blocker = directory.join("blocker");
        fs::write(&blocker, b"not a directory").expect("blocker should be written");
        store.path = blocker.join("profiles.json");

        assert!(matches!(
            store.create(profile("beta", 3_001)),
            Err(ProfileStoreError::Io(_))
        ));
        assert_eq!(store.profiles(), before.as_slice());
        let _ = fs::remove_dir_all(directory);
    }
}
