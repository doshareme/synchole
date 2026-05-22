# Platform Build Tests

Required build targets:

| Platform | Target |
| --- | --- |
| Android | `aarch64-linux-android` |
| Android | `armv7-linux-androideabi` |
| Android | `i686-linux-android` |
| Android | `x86_64-linux-android` |
| iOS device | `aarch64-apple-ios` |
| iOS simulator | `aarch64-apple-ios-sim` |
| macOS | `x86_64-apple-darwin`, `aarch64-apple-darwin` |
| Windows | `x86_64-pc-windows-msvc` |
| Linux | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` |
| Browser/Web Worker | `wasm32-unknown-unknown` |

Recommended commands:

```bash
cargo build --workspace
cargo build -p synchole-bindings --features android --target aarch64-linux-android
cargo build -p synchole-bindings --features apple --target aarch64-apple-ios
cargo build -p synchole-bindings --features c-abi --target x86_64-pc-windows-msvc
cargo build -p synchole-bindings --features wasm --target wasm32-unknown-unknown
```
