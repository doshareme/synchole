# C ABI

The C ABI should remain minimal and stable:

- Opaque handles for runtime, client, storage, and stream objects.
- Explicit destroy functions for every allocated handle.
- Error values returned through a thread-local or caller-provided error buffer.
- No Rust panics across the ABI boundary.
- Background operations represented by cancellable task handles.

The ABI should be generated with `cbindgen` after the raw surface stabilizes.
