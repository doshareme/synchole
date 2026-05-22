use synchole_core::{DeviceId, UserId};
use synchole_identity::{
    AllowListTrustPolicy, DeviceIdentity, KeyAlgorithm, PublicKeyBytes, TrustPolicy,
};

fn identity(user: &str, device: &str) -> DeviceIdentity {
    DeviceIdentity {
        user_id: UserId::from(user),
        device_id: DeviceId::from(device),
        display_name: device.to_owned(),
        signing_key: PublicKeyBytes {
            algorithm: KeyAlgorithm::Ed25519,
            bytes: vec![7; 32],
        },
        encryption_key: PublicKeyBytes {
            algorithm: KeyAlgorithm::Ed25519,
            bytes: vec![8; 32],
        },
        created_at_ms: 42,
        expires_at_ms: None,
    }
}

#[test]
fn allow_list_blocks_unenrolled_devices() {
    let local = identity("user", "phone");
    let remote = identity("user", "laptop");
    let policy = AllowListTrustPolicy::new([DeviceId::from("tablet")]);

    assert!(policy.validate_peer(&local, &remote).is_err());
}
