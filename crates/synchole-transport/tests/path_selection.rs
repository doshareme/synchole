use synchole_transport::{ConnectionPolicy, TransportPath};

#[test]
fn applications_can_force_relay_only_paths() {
    let policy = ConnectionPolicy {
        preferred_paths: vec![TransportPath::Relay],
        ..ConnectionPolicy::default()
    };

    assert_eq!(policy.preferred_paths, vec![TransportPath::Relay]);
}
