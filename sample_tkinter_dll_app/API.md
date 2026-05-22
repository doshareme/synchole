# Synchole DLL API

This document describes how to call the current Windows debug DLL from user
code.

Default DLL path from the repository root:

```text
target/debug/synchole_bindings.dll
```

The sample app in this folder uses Python `ctypes` to load the DLL directly
from that path. The Rust library source and Cargo configuration do not need to
be changed to use this sample.

## Build Requirement

The C ABI exports are available when the bindings crate is built with the
existing `c-abi` feature:

```powershell
cargo build -p synchole-bindings --features c-abi
```

Use a Python runtime with the same architecture as the DLL. For the usual
64-bit Rust Windows toolchain, use 64-bit Python.

## Exported Functions

### `synchole_version_major`

C ABI:

```c
unsigned int synchole_version_major(void);
```

Python `ctypes` binding:

```python
dll.synchole_version_major.argtypes = []
dll.synchole_version_major.restype = ctypes.c_uint
```

Intended usage:

Use this as a quick DLL health check and ABI compatibility probe. If the DLL
loads and this function returns a value, your process can call exported C ABI
functions successfully.

Current expected return:

```text
0
```

Example:

```python
version = dll.synchole_version_major()
print(f"Synchole major version: {version}")
```

### `synchole_storage_mode_is_valid`

C ABI:

```c
bool synchole_storage_mode_is_valid(unsigned int mode);
```

Python `ctypes` binding:

```python
dll.synchole_storage_mode_is_valid.argtypes = [ctypes.c_uint]
dll.synchole_storage_mode_is_valid.restype = ctypes.c_bool
```

Intended usage:

Validate a numeric storage mode before passing it to higher-level Synchole
configuration code. This is useful for UI controls, config files, command-line
arguments, or foreign-language bindings that represent enum values as numbers.

Valid storage mode values:

| Value | Name | Intended Meaning |
| --- | --- | --- |
| `0` | `Binary` | Binary storage mode. |
| `1` | `Sql` | SQL-backed storage mode. |
| `2` | `Document` | Document-oriented storage mode. |

Return values:

| Input | Return |
| --- | --- |
| `0`, `1`, or `2` | `true` |
| Any other unsigned integer | `false` |

Example:

```python
mode = 1
if dll.synchole_storage_mode_is_valid(mode):
    print("Storage mode is supported")
else:
    print("Storage mode is not supported")
```

## Python Usage

Minimal direct `ctypes` example:

```python
import ctypes
import os
from pathlib import Path

repo_root = Path(__file__).resolve().parents[1]
dll_path = repo_root / "target" / "debug" / "synchole_bindings.dll"

os.add_dll_directory(str(dll_path.parent))
dll = ctypes.CDLL(str(dll_path))

dll.synchole_version_major.argtypes = []
dll.synchole_version_major.restype = ctypes.c_uint

dll.synchole_storage_mode_is_valid.argtypes = [ctypes.c_uint]
dll.synchole_storage_mode_is_valid.restype = ctypes.c_bool

print(dll.synchole_version_major())
print(dll.synchole_storage_mode_is_valid(0))
print(dll.synchole_storage_mode_is_valid(999))
```

The sample wrapper in this folder provides the same calls with logging:

```python
from synchole_dll import SyncholeBindings

client = SyncholeBindings()
client.load()

print(client.version_major())
print(client.storage_mode_is_valid(0))
print(client.storage_mode_is_valid(999))
```

## C Usage

Minimal C declarations:

```c
#include <stdbool.h>
#include <stdint.h>

unsigned int synchole_version_major(void);
bool synchole_storage_mode_is_valid(unsigned int mode);
```

Example:

```c
#include <stdbool.h>
#include <stdio.h>

unsigned int synchole_version_major(void);
bool synchole_storage_mode_is_valid(unsigned int mode);

int main(void) {
    printf("Synchole major version: %u\n", synchole_version_major());

    for (unsigned int mode = 0; mode <= 3; mode++) {
        printf("mode %u valid: %s\n",
               mode,
               synchole_storage_mode_is_valid(mode) ? "true" : "false");
    }

    return 0;
}
```

When linking from C or C++, use the import library generated beside the DLL:

```text
target/debug/synchole_bindings.dll.lib
```

## Error Handling Notes

- If loading fails, confirm that `target/debug/synchole_bindings.dll` exists.
- If an exported function is missing, rebuild with `--features c-abi`.
- If Windows reports a bad image or architecture mismatch, use matching 64-bit
  or 32-bit builds for both the DLL and the calling process.
- Keep DLL calls on the C ABI surface. Do not call Rust-internal symbols or
  generated build artifacts from `target/debug/deps`.

## Current API Scope

The current C ABI is intentionally small and stable. It exposes version probing
and storage-mode validation only. Runtime/client/storage handles and background
sync operations are not exported yet.
