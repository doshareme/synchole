# Synchole Todo Sync Tkinter Sample

Two-window Windows/Tkinter sample app that uses two independent SQLite
databases and continuously reconciles todo rows between them.

The sample loads the debug DLL directly from:

```text
target/debug/synchole_bindings.dll
```

It verifies the current C ABI before syncing by calling:

- `synchole_version_major()`
- `synchole_storage_mode_is_valid(1)`

Important: the current DLL does not yet export runtime/client/storage sync
handles. Because this sample does not change library source code, todo row
reconciliation is implemented in Python in `sync_engine.py`. The DLL bridge is
isolated in `synchole_dll.py` so the Python sync adapter can be replaced once
the DLL exposes full sync functions.

## Run

From the repository root:

```powershell
python sample_todo_sync_tkinter_app\main.py
```

Or double-click:

```text
sample_todo_sync_tkinter_app\run_windows.bat
```

The app opens two windows:

- Device A: `sample_todo_sync_tkinter_app/data/device_a.sqlite`
- Device B: `sample_todo_sync_tkinter_app/data/device_b.sqlite`

Add, rename, complete, or delete todos in either window. The sync loop runs
automatically and also runs immediately after edits.

Use **Pause Sync** to stop automatic and edit-triggered sync. While paused,
changes stay only in the window/database where they were made. Use **Resume
Sync** to run reconciliation and converge both databases again.

Each window also has a **Bulk** field and **Populate Here** button. Enter a
count from `1` to `10000` to insert that many todos into only that window's
SQLite database. If sync is paused, the other window remains unchanged until
sync is resumed.

## Build The DLL

If the DLL is missing or does not expose the C ABI functions, rebuild it with
the existing feature flag:

```powershell
cargo build -p synchole-bindings --features c-abi
```

## Smoke Test

Run without opening the GUI:

```powershell
python sample_todo_sync_tkinter_app\main.py --smoke
```

The smoke test creates two temporary SQLite databases, adds todos on both
sides, bulk-populates one side only, syncs them, updates one side, deletes from
the other, and verifies that both databases converge.

## Logs

Runtime logs are written to:

```text
sample_todo_sync_tkinter_app/logs/todo_sync.log
```
