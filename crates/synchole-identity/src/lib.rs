//! Device identity, enrollment, signatures, and trust policy contracts.

use std::collections::BTreeSet;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use synchole_core::{DeviceId, Result, SyncholeError, UserId};
use zeroize::Zeroize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    Ed25519,
    P256,
    HardwareBacked(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKeyBytes {
    pub algorithm: KeyAlgorithm,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureBytes {
    pub algorithm: KeyAlgorithm,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Zeroize)]
#[zeroize(drop)]
pub struct SecretKeyMaterial {
    bytes: Vec<u8>,
}

impl SecretKeyMaterial {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn expose_for_provider(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub display_name: String,
    pub signing_key: PublicKeyBytes,
    pub encryption_key: PublicKeyBytes,
    pub created_at_ms: u64,
    pub expires_at_ms: Option<u64>,
}

impl DeviceIdentity {
    pub fn fingerprint(&self) -> IdentityFingerprint {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.user_id.as_str().as_bytes());
        hasher.update(self.device_id.as_str().as_bytes());
        hasher.update(&self.signing_key.bytes);
        hasher.update(&self.encryption_key.bytes);
        IdentityFingerprint(hasher.finalize().as_bytes().to_vec())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityFingerprint(Vec<u8>);

impl IdentityFingerprint {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnrollmentToken {
    pub user_id: UserId,
    pub inviter_device_id: DeviceId,
    pub expires_at_ms: u64,
    pub signature: SignatureBytes,
}

#[async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn current_device(&self) -> Result<DeviceIdentity>;
    async fn sign(&self, message: &[u8]) -> Result<SignatureBytes>;
    async fn verify(
        &self,
        identity: &DeviceIdentity,
        message: &[u8],
        signature: &SignatureBytes,
    ) -> Result<()>;
}

#[async_trait]
pub trait Keyring: Send + Sync {
    async fn load_or_create_device_identity(
        &self,
        user_id: UserId,
        display_name: String,
    ) -> Result<DeviceIdentity>;

    async fn rotate_device_keys(&self) -> Result<DeviceIdentity>;
    async fn revoke_device(&self, device_id: &DeviceId) -> Result<()>;
}

pub trait TrustPolicy: Send + Sync {
    fn validate_peer(&self, local: &DeviceIdentity, remote: &DeviceIdentity) -> Result<()>;
}

#[derive(Clone, Debug, Default)]
pub struct AllowListTrustPolicy {
    allowed_devices: BTreeSet<DeviceId>,
}

impl AllowListTrustPolicy {
    pub fn new(allowed_devices: impl IntoIterator<Item = DeviceId>) -> Self {
        Self {
            allowed_devices: allowed_devices.into_iter().collect(),
        }
    }

    pub fn insert(&mut self, device_id: DeviceId) {
        self.allowed_devices.insert(device_id);
    }
}

impl TrustPolicy for AllowListTrustPolicy {
    fn validate_peer(&self, local: &DeviceIdentity, remote: &DeviceIdentity) -> Result<()> {
        if local.user_id != remote.user_id {
            return Err(SyncholeError::PermissionDenied(
                "peer belongs to a different user".to_owned(),
            ));
        }

        if !self.allowed_devices.is_empty() && !self.allowed_devices.contains(&remote.device_id) {
            return Err(SyncholeError::PermissionDenied(
                "peer device is not enrolled".to_owned(),
            ));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityAttestation {
    pub identity: DeviceIdentity,
    pub signed_challenge: SignatureBytes,
    pub challenge_nonce: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(user: &str, device: &str) -> DeviceIdentity {
        DeviceIdentity {
            user_id: UserId::from(user),
            device_id: DeviceId::from(device),
            display_name: device.to_owned(),
            signing_key: PublicKeyBytes {
                algorithm: KeyAlgorithm::Ed25519,
                bytes: vec![1; 32],
            },
            encryption_key: PublicKeyBytes {
                algorithm: KeyAlgorithm::Ed25519,
                bytes: vec![2; 32],
            },
            created_at_ms: 1,
            expires_at_ms: None,
        }
    }

    #[test]
    fn rejects_peer_from_different_user() {
        let local = identity("u1", "a");
        let remote = identity("u2", "b");
        let policy = AllowListTrustPolicy::default();

        assert!(policy.validate_peer(&local, &remote).is_err());
    }

    #[test]
    fn accepts_enrolled_peer() {
        let local = identity("u1", "a");
        let remote = identity("u1", "b");
        let policy = AllowListTrustPolicy::new([DeviceId::from("b")]);

        assert!(policy.validate_peer(&local, &remote).is_ok());
    }
}
