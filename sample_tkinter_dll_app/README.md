# Synchole Tkinter DLL Tester

Small Windows-only Python/Tkinter sample app for loading the debug DLL directly
from the repository build output:

```text
target/debug/synchole_bindings.dll
```

It does not change the Rust workspace, Cargo configuration, or library source.

## What It Tests

The current C ABI exports from `synchole_bindings.dll` are:

- `synchole_version_major() -> unsigned int`
- `synchole_storage_mode_is_valid(unsigned int) -> bool`

See `API.md` in this folder for function signatures, intended usage, and
Python/C examples.

The app logs DLL loading, function binding, call inputs, return values, and
pass/fail results to both the Tkinter log panel and:

```text
sample_tkinter_dll_app/logs/synchole_dll_tester.log
```

## Run The GUI

From the repository root:

```powershell
python sample_tkinter_dll_app\main.py
```

Or double-click:

```text
sample_tkinter_dll_app\run_windows.bat
```

## Run A Command-Line Smoke Test

```powershell
python sample_tkinter_dll_app\main.py --smoke
```

The smoke test loads the same DLL path and calls the same functions without
opening the GUI.

## Notes

- Build the DLL first if it is missing:

  ```powershell
  cargo build -p synchole-bindings --features c-abi
  ```

- Use a Python architecture that matches the DLL architecture. For a normal
  64-bit Rust toolchain, use 64-bit Python.
