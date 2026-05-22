from __future__ import annotations

import argparse
import logging
import platform
import sys
import tkinter as tk
from pathlib import Path
from tkinter import messagebox, simpledialog, ttk

from synchole_dll import SyncholeDll, default_dll_path
from sync_engine import TodoSyncCoordinator
from todo_store import TodoItem, TodoStore


APP_DIR = Path(__file__).resolve().parent
DATA_DIR = APP_DIR / "data"
LOG_DIR = APP_DIR / "logs"
LOG_FILE = LOG_DIR / "todo_sync.log"
SYNC_INTERVAL_MS = 750


def configure_logger() -> logging.Logger:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    logger = logging.getLogger("synchole_todo_sync")
    logger.setLevel(logging.DEBUG)
    logger.propagate = False
    logger.handlers.clear()

    formatter = logging.Formatter(
        "%(asctime)s %(levelname)-8s %(message)s", "%Y-%m-%d %H:%M:%S"
    )

    file_handler = logging.FileHandler(LOG_FILE, encoding="utf-8")
    file_handler.setLevel(logging.DEBUG)
    file_handler.setFormatter(formatter)
    logger.addHandler(file_handler)

    stream_handler = logging.StreamHandler()
    stream_handler.setLevel(logging.INFO)
    stream_handler.setFormatter(formatter)
    logger.addHandler(stream_handler)

    return logger


class DeviceWindow:
    def __init__(
        self,
        window: tk.Tk | tk.Toplevel,
        name: str,
        store: TodoStore,
        app: "TodoSyncApp",
    ) -> None:
        self.window = window
        self.name = name
        self.store = store
        self.app = app
        self.title_var = tk.StringVar()
        self.populate_count_var = tk.StringVar(value="100")
        self.status_var = tk.StringVar(value="Ready")
        self.db_var = tk.StringVar(value=str(store.path))
        self.dll_var = tk.StringVar(value="DLL not checked")
        self.pause_button: ttk.Button | None = None

        self._build_ui()
        self.refresh()

    def _build_ui(self) -> None:
        self.window.title(f"Synchole Todo - {self.name}")
        self.window.minsize(620, 600)
        self.window.columnconfigure(0, weight=1)
        self.window.rowconfigure(2, weight=1)

        header = ttk.Frame(self.window, padding=(12, 12, 12, 6))
        header.grid(row=0, column=0, sticky="ew")
        header.columnconfigure(1, weight=1)

        ttk.Label(header, text=self.name, font=("Segoe UI", 15, "bold")).grid(
            row=0, column=0, sticky="w"
        )
        ttk.Label(header, textvariable=self.status_var).grid(row=0, column=1, sticky="e")
        ttk.Label(header, text="DB").grid(row=1, column=0, sticky="w", pady=(8, 0))
        ttk.Label(header, textvariable=self.db_var).grid(
            row=1, column=1, sticky="ew", pady=(8, 0)
        )
        ttk.Label(header, text="DLL").grid(row=2, column=0, sticky="w", pady=(4, 0))
        ttk.Label(header, textvariable=self.dll_var).grid(
            row=2, column=1, sticky="ew", pady=(4, 0)
        )

        editor = ttk.Frame(self.window, padding=(12, 4, 12, 8))
        editor.grid(row=1, column=0, sticky="ew")
        editor.columnconfigure(0, weight=1)

        entry = ttk.Entry(editor, textvariable=self.title_var)
        entry.grid(row=0, column=0, sticky="ew", padx=(0, 8))
        entry.bind("<Return>", lambda _event: self.add_todo())
        ttk.Button(editor, text="Add", command=self.add_todo).grid(row=0, column=1)

        bulk = ttk.Frame(editor)
        bulk.grid(row=1, column=0, columnspan=2, sticky="w", pady=(8, 0))
        ttk.Label(bulk, text="Bulk").grid(row=0, column=0, padx=(0, 6))
        ttk.Spinbox(
            bulk,
            from_=1,
            to=10_000,
            increment=100,
            width=8,
            textvariable=self.populate_count_var,
        ).grid(row=0, column=1, padx=(0, 8))
        ttk.Button(bulk, text="Populate Here", command=self.populate_todos).grid(
            row=0, column=2
        )

        list_frame = ttk.Frame(self.window, padding=(12, 0, 12, 8))
        list_frame.grid(row=2, column=0, sticky="nsew")
        list_frame.columnconfigure(0, weight=1)
        list_frame.rowconfigure(0, weight=1)

        self.tree = ttk.Treeview(
            list_frame,
            columns=("done", "title", "updated"),
            show="headings",
            selectmode="browse",
        )
        self.tree.heading("done", text="Done")
        self.tree.heading("title", text="Todo")
        self.tree.heading("updated", text="Updated")
        self.tree.column("done", width=70, anchor="center", stretch=False)
        self.tree.column("title", width=300, anchor="w")
        self.tree.column("updated", width=150, anchor="center", stretch=False)
        self.tree.grid(row=0, column=0, sticky="nsew")
        self.tree.bind("<Double-1>", lambda _event: self.toggle_selected())

        scroll = ttk.Scrollbar(list_frame, orient="vertical", command=self.tree.yview)
        scroll.grid(row=0, column=1, sticky="ns")
        self.tree.configure(yscrollcommand=scroll.set)

        actions = ttk.Frame(self.window, padding=(12, 0, 12, 12))
        actions.grid(row=3, column=0, sticky="ew")
        actions.columnconfigure(6, weight=1)
        ttk.Button(actions, text="Toggle", command=self.toggle_selected).grid(
            row=0, column=0, padx=(0, 8)
        )
        ttk.Button(actions, text="Rename", command=self.rename_selected).grid(
            row=0, column=1, padx=(0, 8)
        )
        ttk.Button(actions, text="Delete", command=self.delete_selected).grid(
            row=0, column=2, padx=(0, 8)
        )
        ttk.Button(actions, text="Sync Now", command=self.app.sync_now).grid(
            row=0, column=3, padx=(0, 8)
        )
        self.pause_button = ttk.Button(
            actions, text="Pause Sync", command=self.app.toggle_sync_pause
        )
        self.pause_button.grid(row=0, column=4, padx=(0, 8))
        ttk.Button(actions, text="Refresh", command=self.refresh).grid(row=0, column=5)

    def add_todo(self) -> None:
        title = self.title_var.get().strip()
        if not title:
            self.status_var.set("Type a todo first")
            return
        self.store.add(title)
        self.title_var.set("")
        self.status_var.set("Added locally")
        self.app.logger.info("%s added todo: %s", self.name, title)
        self.app.after_local_change(self)

    def toggle_selected(self) -> None:
        item = self.selected_item()
        if item is None:
            self.status_var.set("Select a todo first")
            return
        self.store.set_completed(item.id, not item.completed)
        self.status_var.set("Updated locally")
        self.app.logger.info("%s toggled todo %s", self.name, item.id)
        self.app.after_local_change(self)

    def rename_selected(self) -> None:
        item = self.selected_item()
        if item is None:
            self.status_var.set("Select a todo first")
            return
        new_title = simpledialog.askstring(
            "Rename todo",
            "Todo title",
            initialvalue=item.title,
            parent=self.window,
        )
        if new_title is None:
            return
        new_title = new_title.strip()
        if not new_title:
            self.status_var.set("Title cannot be empty")
            return
        self.store.rename(item.id, new_title)
        self.status_var.set("Renamed locally")
        self.app.logger.info("%s renamed todo %s", self.name, item.id)
        self.app.after_local_change(self)

    def delete_selected(self) -> None:
        item = self.selected_item()
        if item is None:
            self.status_var.set("Select a todo first")
            return
        self.store.delete(item.id)
        self.status_var.set("Deleted locally")
        self.app.logger.info("%s deleted todo %s", self.name, item.id)
        self.app.after_local_change(self)

    def populate_todos(self) -> None:
        try:
            count = int(self.populate_count_var.get())
        except ValueError:
            self.status_var.set("Bulk count must be a number")
            return

        if count < 1 or count > 10_000:
            self.status_var.set("Bulk count must be 1-10000")
            return

        inserted = self.store.bulk_add(count, f"{self.name} todo")
        self.status_var.set(f"Populated {inserted} locally")
        self.app.logger.info("%s populated %d todos locally", self.name, inserted)
        self.app.after_local_change(self)

    def selected_item(self) -> TodoItem | None:
        selected = self.tree.selection()
        if not selected:
            return None
        item_id = selected[0]
        return self.store.get(item_id)

    def refresh(self) -> None:
        selected = self.tree.selection()
        selected_id = selected[0] if selected else None
        rows = self.tree.get_children()
        if rows:
            self.tree.delete(*rows)

        for item in self.store.list_active():
            done = "yes" if item.completed else "no"
            self.tree.insert(
                "",
                "end",
                iid=item.id,
                values=(done, item.title, item.updated_at_text),
            )

        if selected_id and self.tree.exists(selected_id):
            self.tree.selection_set(selected_id)

    def set_sync_status(self, text: str) -> None:
        self.status_var.set(text)

    def set_dll_status(self, text: str) -> None:
        self.dll_var.set(text)

    def set_sync_paused(self, paused: bool) -> None:
        if self.pause_button is not None:
            self.pause_button.configure(text="Resume Sync" if paused else "Pause Sync")


class TodoSyncApp:
    def __init__(self, dll_path: Path | None = None) -> None:
        DATA_DIR.mkdir(parents=True, exist_ok=True)
        self.logger = configure_logger()
        self.logger.info("Starting Synchole Todo Sync sample")
        self.logger.info("Python: %s (%s)", sys.version.split()[0], platform.architecture()[0])
        self.logger.info("Log file: %s", LOG_FILE)

        self.left_store = TodoStore(DATA_DIR / "device_a.sqlite", "device-a", self.logger)
        self.right_store = TodoStore(DATA_DIR / "device_b.sqlite", "device-b", self.logger)
        self.left_store.initialize()
        self.right_store.initialize()

        self.dll = SyncholeDll(dll_path or default_dll_path(), self.logger)
        self.coordinator = TodoSyncCoordinator(
            left=self.left_store,
            right=self.right_store,
            dll=self.dll,
            logger=self.logger,
        )

        self.sync_paused = False
        self.root = tk.Tk()
        self.root.protocol("WM_DELETE_WINDOW", self.close)
        self.right_toplevel = tk.Toplevel(self.root)
        self.right_toplevel.protocol("WM_DELETE_WINDOW", self.close)

        self.left_window = DeviceWindow(self.root, "Device A", self.left_store, self)
        self.right_window = DeviceWindow(
            self.right_toplevel, "Device B", self.right_store, self
        )
        self.root.geometry("640x590+80+80")
        self.right_toplevel.geometry("640x590+760+80")

        self._sync_in_progress = False
        self.sync_now()
        self.schedule_sync()

    def schedule_sync(self) -> None:
        self.root.after(SYNC_INTERVAL_MS, self.sync_tick)

    def sync_tick(self) -> None:
        if not self.sync_paused:
            self.sync_now(show_errors=False)
        self.schedule_sync()

    def sync_now(self, show_errors: bool = True, force: bool = False) -> None:
        if self.sync_paused and not force:
            self.set_all_statuses("Sync paused")
            return
        if self._sync_in_progress:
            return
        self._sync_in_progress = True
        try:
            report = self.coordinator.sync_once()
        except Exception as exc:
            self.logger.exception("Sync failed: %s", exc)
            self.left_window.set_sync_status("Sync failed")
            self.right_window.set_sync_status("Sync failed")
            self.left_window.set_dll_status("DLL check failed")
            self.right_window.set_dll_status("DLL check failed")
            if show_errors:
                messagebox.showerror("Sync failed", str(exc), parent=self.root)
            return
        finally:
            self._sync_in_progress = False

        self.left_window.refresh()
        self.right_window.refresh()

        status = (
            f"Synced {report.copied_to_left + report.copied_to_right} change(s) "
            f"in {report.duration_ms} ms"
        )
        dll_status = f"DLL ok, version {report.dll_version}, SQL mode valid"
        self.left_window.set_sync_status(status)
        self.right_window.set_sync_status(status)
        self.left_window.set_dll_status(dll_status)
        self.right_window.set_dll_status(dll_status)

    def after_local_change(self, source_window: DeviceWindow) -> None:
        if self.sync_paused:
            source_window.refresh()
            self.set_all_statuses("Sync paused; local changes queued")
            self.logger.info("%s change queued while sync is paused", source_window.name)
            return
        self.sync_now()

    def toggle_sync_pause(self) -> None:
        self.sync_paused = not self.sync_paused
        self.left_window.set_sync_paused(self.sync_paused)
        self.right_window.set_sync_paused(self.sync_paused)

        if self.sync_paused:
            self.set_all_statuses("Sync paused")
            self.logger.info("Sync paused")
            return

        self.set_all_statuses("Sync resumed")
        self.logger.info("Sync resumed")
        self.sync_now(force=True)

    def set_all_statuses(self, text: str) -> None:
        self.left_window.set_sync_status(text)
        self.right_window.set_sync_status(text)

    def run(self) -> int:
        self.root.mainloop()
        return 0

    def close(self) -> None:
        self.logger.info("Closing Todo Sync sample")
        self.left_store.close()
        self.right_store.close()
        self.root.destroy()


def run_smoke_test(dll_path: Path | None = None) -> int:
    logger = configure_logger()
    logger.info("Running command-line todo sync smoke test")
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    left_path = DATA_DIR / "smoke_device_a.sqlite"
    right_path = DATA_DIR / "smoke_device_b.sqlite"
    for path in (left_path, right_path):
        if path.exists():
            path.unlink()

    left = TodoStore(left_path, "smoke-a", logger)
    right = TodoStore(right_path, "smoke-b", logger)
    left.initialize()
    right.initialize()

    coordinator = TodoSyncCoordinator(
        left=left,
        right=right,
        dll=SyncholeDll(dll_path or default_dll_path(), logger),
        logger=logger,
    )

    try:
        left_item = left.add("Write DLL todo sample")
        right_item = right.add("Verify independent SQLite sync")
        report = coordinator.sync_once()
        logger.info("Initial sync report: %s", report)

        assert left.count_active() == 2
        assert right.count_active() == 2

        inserted = left.bulk_add(25, "Smoke bulk")
        assert inserted == 25
        assert left.count_active() == 27
        assert right.count_active() == 2
        report = coordinator.sync_once()
        logger.info("Bulk populate sync report: %s", report)

        assert left.count_active() == 27
        assert right.count_active() == 27

        left.set_completed(right_item.id, True)
        right.rename(left_item.id, "Write Tkinter DLL todo sample")
        report = coordinator.sync_once()
        logger.info("Update sync report: %s", report)

        assert left.get(right_item.id).completed is True
        assert right.get(right_item.id).completed is True
        assert left.get(left_item.id).title == "Write Tkinter DLL todo sample"
        assert right.get(left_item.id).title == "Write Tkinter DLL todo sample"

        right.delete(left_item.id)
        report = coordinator.sync_once()
        logger.info("Delete sync report: %s", report)

        assert left.count_active() == 26
        assert right.count_active() == 26
    except Exception as exc:
        logger.exception("Smoke test failed: %s", exc)
        return 1
    finally:
        left.close()
        right.close()

    logger.info("Todo sync smoke test passed")
    return 0


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Two-window Synchole todo sync sample")
    parser.add_argument("--smoke", action="store_true", help="run a CLI smoke test")
    parser.add_argument("--dll", type=Path, help="override DLL path")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    if args.smoke:
        return run_smoke_test(args.dll)
    return TodoSyncApp(args.dll).run()


if __name__ == "__main__":
    raise SystemExit(main())
