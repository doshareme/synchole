# Android Binding

Recommended packaging:

- Build Rust with `cargo ndk` for `arm64-v8a`, `armeabi-v7a`, `x86`, and `x86_64`.
- Publish an AAR containing the native libraries and Kotlin wrappers.
- Store device identity keys in Android Keystore when hardware-backed keys are available.
- Run sync work through a foreground service or WorkManager depending on battery policy.
- Expose Kotlin coroutines for async SDK calls and stream sync events through `Flow`.

Public Kotlin shape:

```kotlin
class SyncholeClient private constructor(...)

data class SyncholeConfig(
    val storagePath: String,
    val storageMode: StorageMode,
    val relayRegions: List<String>
)

suspend fun SyncholeClient.syncOnce(): SyncReport
```
