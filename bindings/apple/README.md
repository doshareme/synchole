# Apple Binding

Recommended packaging:

- Use UniFFI for Swift-friendly SDK APIs.
- Build an XCFramework for iOS device, iOS simulator, and macOS.
- Store long-term identity keys in Keychain or Secure Enclave where available.
- Integrate with `BGTaskScheduler` on iOS for battery-aware background sync.
- Use Network.framework adapters for path monitoring and peer connectivity.

Public Swift shape:

```swift
let client = try await SyncholeClient.open(config)
try await client.putDocument(collection: "notes", id: "n1", document: note)
let report = try await client.syncOnce()
```
