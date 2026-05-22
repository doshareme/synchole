use serde_json::json;
use synchole_core::{Timestamp, VersionVector};
use synchole_storage::{ObjectKey, StoragePayload, StoredObject};
use synchole_sync::{Conflict, ConflictResolver, KeepBothResolver, Resolution};

fn object(device: &str) -> StoredObject {
    let mut version = VersionVector::new();
    version.increment(device.into());
    StoredObject::new(
        "docs".into(),
        "doc-1".into(),
        StoragePayload::Document(json!({"body": device})),
        version,
        Timestamp {
            wall_time_ms: 10,
            logical: 0,
        },
    )
}

#[tokio::test]
async fn keep_both_resolver_preserves_both_versions() {
    let resolver = KeepBothResolver;
    let resolution = resolver
        .resolve(Conflict {
            key: ObjectKey::new("docs".into(), "doc-1".into()),
            local: object("phone"),
            remote: object("laptop"),
        })
        .await
        .expect("resolve");

    match resolution {
        Resolution::KeepBoth(objects) => assert_eq!(objects.len(), 2),
        other => panic!("expected keep-both resolution, got {other:?}"),
    }
}
