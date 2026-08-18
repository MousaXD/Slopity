use crate::{validate_profile, ServerId, ServerProfile, ValidationSeverity};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRecoveryNotice {
    pub code: String,
    pub message: String,
}

impl ProfileRecoveryNotice {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProfileStoreError {
    #[error("profile storage I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("profile document JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("profile document schema metadata is invalid: {0}")]
    InvalidSchema(String),
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
    #[error("no free profile port is available")]
    NoAvailablePort,
    #[error(
        "profile storage is unrecoverable: primary={primary_error}; temporary={temporary_error}; backup={backup_error}"
    )]
    Unrecoverable {
        primary_error: String,
        temporary_error: String,
        backup_error: String,
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
    recovery_notices: Vec<ProfileRecoveryNotice>,
}

impl ProfileStore {
    pub fn load_or_create(
        path: impl Into<PathBuf>,
        seed_profiles: Vec<ServerProfile>,
    ) -> Result<Self, ProfileStoreError> {
        let path = path.into();
        let temporary = temporary_path(&path);
        let backup = backup_path(&path);
        let mut notices = Vec::new();

        match inspect_candidate(&path)? {
            CandidateState::Valid(loaded) => {
                if temporary.exists() {
                    fs::remove_file(&temporary)?;
                    sync_parent_directory(&path)?;
                    notices.push(ProfileRecoveryNotice::new(
                        "stale-temporary-removed",
                        "A stale temporary profile file was removed because the committed primary file was valid.",
                    ));
                }

                let document = loaded.document;
                if let Some(from_version) = loaded.migrated_from {
                    write_document(&path, &document)?;
                    notices.push(migration_notice(from_version));
                }

                Ok(Self {
                    path,
                    document,
                    recovery_notices: notices,
                })
            }
            CandidateState::Missing => {
                recover_missing_primary(path, temporary, backup, seed_profiles, notices)
            }
            CandidateState::Invalid(primary_error) => {
                recover_corrupt_primary(path, temporary, backup, primary_error, notices)
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn profiles(&self) -> &[ServerProfile] {
        &self.document.profiles
    }

    pub fn recovery_notices(&self) -> &[ProfileRecoveryNotice] {
        &self.recovery_notices
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
        cloned.port = self
            .next_available_port(cloned.port)
            .ok_or(ProfileStoreError::NoAvailablePort)?;
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

    fn next_available_port(&self, after: u16) -> Option<u16> {
        let reserved_ports = self
            .document
            .profiles
            .iter()
            .map(|profile| profile.port)
            .collect::<HashSet<_>>();
        let mut candidate = after;

        for _ in 0..u16::MAX {
            candidate = if candidate == u16::MAX {
                1
            } else {
                candidate + 1
            };
            if !reserved_ports.contains(&candidate) {
                return Some(candidate);
            }
        }

        None
    }

    fn commit_profiles(&mut self, profiles: Vec<ServerProfile>) -> Result<(), ProfileStoreError> {
        let document = ProfileDocument::new(profiles)?;
        write_document(&self.path, &document)?;
        self.document = document;
        Ok(())
    }
}

#[derive(Debug)]
struct LoadedDocument {
    document: ProfileDocument,
    migrated_from: Option<u32>,
}

#[derive(Debug)]
enum CandidateState {
    Missing,
    Valid(LoadedDocument),
    Invalid(String),
}

fn inspect_candidate(path: &Path) -> Result<CandidateState, ProfileStoreError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(CandidateState::Missing)
        }
        Err(error) => return Err(ProfileStoreError::Io(error)),
    };

    match decode_and_migrate_document(&bytes) {
        Ok(document) => Ok(CandidateState::Valid(document)),
        Err(error @ ProfileStoreError::UnsupportedSchema { .. }) => Err(error),
        Err(error) => Ok(CandidateState::Invalid(error.to_string())),
    }
}

fn recover_missing_primary(
    path: PathBuf,
    temporary: PathBuf,
    backup: PathBuf,
    seed_profiles: Vec<ServerProfile>,
    mut notices: Vec<ProfileRecoveryNotice>,
) -> Result<ProfileStore, ProfileStoreError> {
    let temporary_state = inspect_candidate(&temporary)?;
    match temporary_state {
        CandidateState::Valid(loaded) => {
            let backup_was_present = backup.exists();
            promote_temporary(&path, &temporary)?;
            notices.push(ProfileRecoveryNotice::new(
                if backup_was_present {
                    "interrupted-replacement-recovered"
                } else {
                    "temporary-primary-recovered"
                },
                if backup_was_present {
                    "Recovered the intended replacement profile document from the temporary file after an interrupted replacement; the previous backup was retained."
                } else {
                    "Recovered the profile document from a complete temporary file because the primary file was missing."
                },
            ));

            let document = loaded.document;
            if let Some(from_version) = loaded.migrated_from {
                write_document(&path, &document)?;
                notices.push(migration_notice(from_version));
            }

            return Ok(ProfileStore {
                path,
                document,
                recovery_notices: notices,
            });
        }
        CandidateState::Invalid(temporary_error) => {
            return recover_missing_primary_from_backup(
                path,
                temporary,
                backup,
                seed_profiles,
                notices,
                temporary_error,
            );
        }
        CandidateState::Missing => {}
    }

    match inspect_candidate(&backup)? {
        CandidateState::Valid(loaded) => {
            install_recovered_document(&path, &loaded.document, false)?;
            notices.push(ProfileRecoveryNotice::new(
                "backup-primary-recovered",
                "Restored the profile document from the backup because the primary file was missing.",
            ));
            if let Some(from_version) = loaded.migrated_from {
                notices.push(migration_notice(from_version));
            }
            Ok(ProfileStore {
                path,
                document: loaded.document,
                recovery_notices: notices,
            })
        }
        CandidateState::Invalid(backup_error) => Err(ProfileStoreError::Unrecoverable {
            primary_error: "missing".into(),
            temporary_error: "missing".into(),
            backup_error,
        }),
        CandidateState::Missing => {
            let document = ProfileDocument::new(seed_profiles)?;
            write_document(&path, &document)?;
            Ok(ProfileStore {
                path,
                document,
                recovery_notices: notices,
            })
        }
    }
}

fn recover_missing_primary_from_backup(
    path: PathBuf,
    _temporary: PathBuf,
    backup: PathBuf,
    _seed_profiles: Vec<ServerProfile>,
    mut notices: Vec<ProfileRecoveryNotice>,
    temporary_error: String,
) -> Result<ProfileStore, ProfileStoreError> {
    match inspect_candidate(&backup)? {
        CandidateState::Valid(loaded) => {
            install_recovered_document(&path, &loaded.document, false)?;
            notices.push(ProfileRecoveryNotice::new(
                "invalid-temporary-discarded",
                format!(
                    "Discarded an invalid temporary profile file while restoring the valid backup: {temporary_error}"
                ),
            ));
            notices.push(ProfileRecoveryNotice::new(
                "backup-primary-recovered",
                "Restored the profile document from the backup because the primary file was missing.",
            ));
            if let Some(from_version) = loaded.migrated_from {
                notices.push(migration_notice(from_version));
            }
            Ok(ProfileStore {
                path,
                document: loaded.document,
                recovery_notices: notices,
            })
        }
        CandidateState::Invalid(backup_error) => Err(ProfileStoreError::Unrecoverable {
            primary_error: "missing".into(),
            temporary_error,
            backup_error,
        }),
        CandidateState::Missing => Err(ProfileStoreError::Unrecoverable {
            primary_error: "missing".into(),
            temporary_error,
            backup_error: "missing".into(),
        }),
    }
}

fn recover_corrupt_primary(
    path: PathBuf,
    temporary: PathBuf,
    backup: PathBuf,
    primary_error: String,
    mut notices: Vec<ProfileRecoveryNotice>,
) -> Result<ProfileStore, ProfileStoreError> {
    let backup_state = inspect_candidate(&backup)?;
    match backup_state {
        CandidateState::Valid(loaded) => {
            let temporary_was_present = temporary.exists();
            install_recovered_document(&path, &loaded.document, true)?;
            if temporary_was_present {
                notices.push(ProfileRecoveryNotice::new(
                    "temporary-discarded-during-backup-recovery",
                    "Discarded the uncommitted temporary file while restoring the last known-good backup.",
                ));
            }
            notices.push(ProfileRecoveryNotice::new(
                "corrupt-primary-recovered-from-backup",
                format!(
                    "The primary profile file was invalid and was preserved as a .corrupt file before the valid backup was restored: {primary_error}"
                ),
            ));
            if let Some(from_version) = loaded.migrated_from {
                notices.push(migration_notice(from_version));
            }
            return Ok(ProfileStore {
                path,
                document: loaded.document,
                recovery_notices: notices,
            });
        }
        CandidateState::Invalid(backup_error) => {
            return recover_corrupt_primary_from_temporary(
                path,
                temporary,
                primary_error,
                backup_error,
                notices,
            );
        }
        CandidateState::Missing => {}
    }

    recover_corrupt_primary_from_temporary(
        path,
        temporary,
        primary_error,
        "missing".into(),
        notices,
    )
}

fn recover_corrupt_primary_from_temporary(
    path: PathBuf,
    temporary: PathBuf,
    primary_error: String,
    backup_error: String,
    mut notices: Vec<ProfileRecoveryNotice>,
) -> Result<ProfileStore, ProfileStoreError> {
    match inspect_candidate(&temporary)? {
        CandidateState::Valid(loaded) => {
            promote_temporary_over_corrupt_primary(&path, &temporary)?;
            notices.push(ProfileRecoveryNotice::new(
                "corrupt-primary-recovered-from-temporary",
                format!(
                    "The primary profile file was invalid and no usable backup existed; a complete temporary file was promoted and the corrupt primary was preserved as a .corrupt file: {primary_error}"
                ),
            ));
            let document = loaded.document;
            if let Some(from_version) = loaded.migrated_from {
                write_document(&path, &document)?;
                notices.push(migration_notice(from_version));
            }
            Ok(ProfileStore {
                path,
                document,
                recovery_notices: notices,
            })
        }
        CandidateState::Invalid(temporary_error) => Err(ProfileStoreError::Unrecoverable {
            primary_error,
            temporary_error,
            backup_error,
        }),
        CandidateState::Missing => Err(ProfileStoreError::Unrecoverable {
            primary_error,
            temporary_error: "missing".into(),
            backup_error,
        }),
    }
}

fn migration_notice(from_version: u32) -> ProfileRecoveryNotice {
    ProfileRecoveryNotice::new(
        "profile-schema-migrated",
        format!(
            "Migrated the profile document from schema v{from_version} to v{PROFILE_SCHEMA_VERSION}."
        ),
    )
}

fn decode_and_migrate_document(bytes: &[u8]) -> Result<LoadedDocument, ProfileStoreError> {
    let mut value: Value = serde_json::from_slice(bytes)?;
    let original_version = schema_version(&value)?;

    if original_version > PROFILE_SCHEMA_VERSION {
        return Err(ProfileStoreError::UnsupportedSchema {
            found: original_version,
            supported: PROFILE_SCHEMA_VERSION,
        });
    }

    let mut version = original_version;
    while version < PROFILE_SCHEMA_VERSION {
        match version {
            0 => {
                set_schema_version(&mut value, 1)?;
                version = 1;
            }
            found => {
                return Err(ProfileStoreError::UnsupportedSchema {
                    found,
                    supported: PROFILE_SCHEMA_VERSION,
                });
            }
        }
    }

    let document: ProfileDocument = serde_json::from_value(value)?;
    validate_document(&document)?;
    Ok(LoadedDocument {
        document,
        migrated_from: (original_version != PROFILE_SCHEMA_VERSION).then_some(original_version),
    })
}

fn schema_version(value: &Value) -> Result<u32, ProfileStoreError> {
    let version = value
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            ProfileStoreError::InvalidSchema(
                "schemaVersion must be present and contain a non-negative integer".into(),
            )
        })?;
    u32::try_from(version).map_err(|_| {
        ProfileStoreError::InvalidSchema("schemaVersion is larger than a 32-bit version".into())
    })
}

fn set_schema_version(value: &mut Value, version: u32) -> Result<(), ProfileStoreError> {
    let object = value.as_object_mut().ok_or_else(|| {
        ProfileStoreError::InvalidSchema("profile document root must be a JSON object".into())
    })?;
    object.insert("schemaVersion".into(), Value::from(version));
    Ok(())
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
    ensure_parent_directory(path)?;

    let temporary = temporary_path(path);
    let backup = backup_path(path);
    if temporary.exists() {
        fs::remove_file(&temporary)?;
        sync_parent_directory(path)?;
    }

    write_temporary_document(&temporary, document)?;
    sync_parent_directory(path)?;

    if path.exists() {
        fs::copy(path, &backup)?;
        sync_file(&backup)?;
        sync_parent_directory(path)?;
    }

    match fs::rename(&temporary, path) {
        Ok(()) => {
            sync_parent_directory(path)?;
            Ok(())
        }
        Err(_) if path.exists() => {
            fs::remove_file(path)?;
            sync_parent_directory(path)?;
            match fs::rename(&temporary, path) {
                Ok(()) => {
                    sync_parent_directory(path)?;
                    Ok(())
                }
                Err(write_error) => match restore_backup_copy(path, &backup) {
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

fn install_recovered_document(
    path: &Path,
    document: &ProfileDocument,
    preserve_corrupt_primary: bool,
) -> Result<(), ProfileStoreError> {
    ensure_parent_directory(path)?;
    let temporary = temporary_path(path);
    if temporary.exists() {
        fs::remove_file(&temporary)?;
        sync_parent_directory(path)?;
    }
    write_temporary_document(&temporary, document)?;
    sync_parent_directory(path)?;

    if preserve_corrupt_primary && path.exists() {
        preserve_corrupt_file(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
        sync_parent_directory(path)?;
    }

    fs::rename(&temporary, path)?;
    sync_parent_directory(path)?;
    Ok(())
}

fn promote_temporary(path: &Path, temporary: &Path) -> Result<(), ProfileStoreError> {
    ensure_parent_directory(path)?;
    sync_file(temporary)?;
    fs::rename(temporary, path)?;
    sync_parent_directory(path)?;
    Ok(())
}

fn promote_temporary_over_corrupt_primary(
    path: &Path,
    temporary: &Path,
) -> Result<(), ProfileStoreError> {
    ensure_parent_directory(path)?;
    sync_file(temporary)?;
    preserve_corrupt_file(path)?;
    fs::rename(temporary, path)?;
    sync_parent_directory(path)?;
    Ok(())
}

fn preserve_corrupt_file(path: &Path) -> Result<(), ProfileStoreError> {
    let corrupt = corrupt_path(path);
    if corrupt.exists() {
        fs::remove_file(&corrupt)?;
    }
    fs::rename(path, corrupt)?;
    sync_parent_directory(path)?;
    Ok(())
}

fn write_temporary_document(
    temporary: &Path,
    document: &ProfileDocument,
) -> Result<(), ProfileStoreError> {
    let mut bytes = serde_json::to_vec_pretty(document)?;
    bytes.push(b'\n');

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn restore_backup_copy(path: &Path, backup: &Path) -> io::Result<()> {
    fs::copy(backup, path)?;
    sync_file(path)?;
    sync_parent_directory(path)
}

fn sync_file(path: &Path) -> io::Result<()> {
    OpenOptions::new().read(true).open(path)?.sync_all()
}

fn ensure_parent_directory(path: &Path) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    path.with_extension("tmp")
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("bak")
}

fn corrupt_path(path: &Path) -> PathBuf {
    path.with_extension("corrupt")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NetworkScope, RuntimeKind};
    use serde_json::json;
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

    fn document_bytes(schema_version: u32, profiles: Vec<ServerProfile>) -> Vec<u8> {
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": schema_version,
            "profiles": profiles,
        }))
        .expect("fixture should serialize")
    }

    fn write_fixture(path: &Path, schema_version: u32, profiles: Vec<ServerProfile>) {
        fs::write(path, document_bytes(schema_version, profiles))
            .expect("fixture should be written");
    }

    fn has_notice(store: &ProfileStore, code: &str) -> bool {
        store
            .recovery_notices()
            .iter()
            .any(|notice| notice.code == code)
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
    fn stale_temporary_is_removed_when_primary_is_valid() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        write_fixture(&path, PROFILE_SCHEMA_VERSION, vec![profile("alpha", 3_000)]);
        write_fixture(
            &temporary_path(&path),
            PROFILE_SCHEMA_VERSION,
            vec![profile("beta", 3_001)],
        );

        let store = ProfileStore::load_or_create(&path, Vec::new()).expect("primary should win");
        assert_eq!(store.profiles()[0].id.0, "alpha");
        assert!(!temporary_path(&path).exists());
        assert!(has_notice(&store, "stale-temporary-removed"));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn missing_primary_is_restored_from_valid_backup() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        write_fixture(
            &backup_path(&path),
            PROFILE_SCHEMA_VERSION,
            vec![profile("alpha", 3_000)],
        );

        let store = ProfileStore::load_or_create(&path, Vec::new()).expect("backup should recover");
        assert_eq!(store.profiles()[0].id.0, "alpha");
        assert!(path.exists());
        assert!(backup_path(&path).exists());
        assert!(has_notice(&store, "backup-primary-recovered"));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn corrupt_primary_is_restored_from_valid_backup() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        fs::write(&path, b"not-json").expect("corrupt primary should be written");
        write_fixture(
            &backup_path(&path),
            PROFILE_SCHEMA_VERSION,
            vec![profile("alpha", 3_000)],
        );

        let store = ProfileStore::load_or_create(&path, Vec::new()).expect("backup should recover");
        assert_eq!(store.profiles()[0].id.0, "alpha");
        assert!(corrupt_path(&path).exists());
        assert!(has_notice(&store, "corrupt-primary-recovered-from-backup"));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn corrupt_primary_and_corrupt_backup_are_not_replaced_with_seed_data() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        fs::write(&path, b"bad-primary").expect("primary fixture should be written");
        fs::write(backup_path(&path), b"bad-backup").expect("backup fixture should be written");

        assert!(matches!(
            ProfileStore::load_or_create(&path, vec![profile("seed", 3_000)]),
            Err(ProfileStoreError::Unrecoverable { .. })
        ));
        assert_eq!(
            fs::read(&path).expect("primary should remain"),
            b"bad-primary"
        );
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn interrupted_replacement_prefers_complete_temporary_over_backup() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        write_fixture(
            &temporary_path(&path),
            PROFILE_SCHEMA_VERSION,
            vec![profile("new", 3_001)],
        );
        write_fixture(
            &backup_path(&path),
            PROFILE_SCHEMA_VERSION,
            vec![profile("old", 3_000)],
        );

        let store =
            ProfileStore::load_or_create(&path, Vec::new()).expect("temporary should recover");
        assert_eq!(store.profiles()[0].id.0, "new");
        assert!(path.exists());
        assert!(!temporary_path(&path).exists());
        assert!(backup_path(&path).exists());
        assert!(has_notice(&store, "interrupted-replacement-recovered"));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn successful_normal_write_keeps_previous_backup_and_reloads() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        let mut store =
            ProfileStore::load_or_create(&path, Vec::new()).expect("store should be created");
        store
            .create(profile("alpha", 3_000))
            .expect("profile should be persisted");

        assert!(path.exists());
        assert!(backup_path(&path).exists());
        let reopened =
            ProfileStore::load_or_create(&path, Vec::new()).expect("store should reload");
        assert_eq!(reopened.profiles().len(), 1);
        assert_eq!(reopened.profiles()[0].id.0, "alpha");
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn schema_v0_fixture_migrates_to_current_schema_without_profile_loss() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        write_fixture(&path, 0, vec![profile("alpha", 3_000)]);

        let store = ProfileStore::load_or_create(&path, Vec::new()).expect("v0 should migrate");
        assert_eq!(store.profiles()[0].id.0, "alpha");
        assert!(has_notice(&store, "profile-schema-migrated"));
        let document: ProfileDocument =
            serde_json::from_slice(&fs::read(&path).expect("migrated document should exist"))
                .expect("migrated document should parse");
        assert_eq!(document.schema_version, PROFILE_SCHEMA_VERSION);
        assert_eq!(document.profiles[0].id.0, "alpha");
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn future_schema_is_rejected_without_falling_back() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        write_fixture(&path, 99, Vec::new());
        write_fixture(
            &backup_path(&path),
            PROFILE_SCHEMA_VERSION,
            vec![profile("older", 3_000)],
        );

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
        let cloned = store
            .clone_profile(
                &ServerId("alpha".into()),
                ServerId("beta".into()),
                "Beta clone".into(),
            )
            .expect("profile should be cloned");
        assert_eq!(cloned.port, 3_002);
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
        assert_eq!(reopened.profiles()[0].port, 3_002);
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
