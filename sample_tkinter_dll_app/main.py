from __future__ import annotations

import argparse
import logging
import os
import platform
import sys
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk

from synchole_dll import SyncholeBindings, default_dll_path


APP_DIR = Path(__file__).resolve().parent
LOG_DIR = APP_DIR / "logs"
LOG_FILE = LOG_DIR / "synchole_dll_tester.log"


class TkTextHandler(logging.Handler):
    def __init__(self, widget: tk.Text) -> None:
        super().__init__()
        self.widget = widget

    def emit(self, record: logging.LogRecord) -> None:
        message = self.format(record)

        def append() -> None:
            self.widget.configure(state="normal")
            self.widget.insert("end", message + "\n")
            self.widget.see("end")
            self.widget.configure(state="disabled")

        self.widget.after(0, append)


def configure_logger() -> logging.Logger:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    logger = logging.getLogger("synchole_dll_tester")
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


class DllTesterApp(tk.Tk):
    def __init__(self, dll_path: Path | None = None) -> None:
        super().__init__()
        self.title("Synchole DLL Tester")
        self.minsize(900, 620)

        self.logger = configure_logger()
        self.client: SyncholeBindings | None = None
        self.dll_path_var = tk.StringVar(value=str(dll_path or default_dll_path()))
        self.status_var = tk.StringVar(value="DLL not loaded")
        self.mode_var = tk.StringVar(value="0")

        self._build_ui()
        self._attach_log_handler()
        self._log_startup()

    def _build_ui(self) -> None:
        self.columnconfigure(0, weight=1)
        self.rowconfigure(3, weight=1)

        toolbar = ttk.Frame(self, padding=(12, 12, 12, 6))
        toolbar.grid(row=0, column=0, sticky="ew")
        toolbar.columnconfigure(1, weight=1)

        ttk.Label(toolbar, text="DLL").grid(row=0, column=0, sticky="w")
        dll_entry = ttk.Entry(toolbar, textvariable=self.dll_path_var)
        dll_entry.grid(row=0, column=1, sticky="ew", padx=(8, 8))
        ttk.Button(toolbar, text="Browse", command=self.browse_dll).grid(
            row=0, column=2, sticky="e"
        )

        status_bar = ttk.Frame(self, padding=(12, 0, 12, 6))
        status_bar.grid(row=1, column=0, sticky="ew")
        status_bar.columnconfigure(1, weight=1)
        ttk.Label(status_bar, text="Status").grid(row=0, column=0, sticky="w")
        ttk.Label(status_bar, textvariable=self.status_var).grid(
            row=0, column=1, sticky="w", padx=(8, 0)
        )

        actions = ttk.Frame(self, padding=(12, 0, 12, 8))
        actions.grid(row=2, column=0, sticky="ew")
        for column in range(7):
            actions.columnconfigure(column, weight=0)
        actions.columnconfigure(7, weight=1)

        ttk.Button(actions, text="Load DLL", command=self.load_dll).grid(
            row=0, column=0, padx=(0, 8)
        )
        ttk.Button(actions, text="Run All Tests", command=self.run_all_tests).grid(
            row=0, column=1, padx=(0, 8)
        )
        ttk.Button(actions, text="Version", command=self.call_version).grid(
            row=0, column=2, padx=(0, 8)
        )

        ttk.Label(actions, text="Mode").grid(row=0, column=3, padx=(8, 4))
        ttk.Spinbox(
            actions,
            from_=0,
            to=999,
            increment=1,
            width=8,
            textvariable=self.mode_var,
        ).grid(row=0, column=4, padx=(0, 8))
        ttk.Button(actions, text="Validate Mode", command=self.validate_mode).grid(
            row=0, column=5, padx=(0, 8)
        )
        ttk.Button(actions, text="Clear Log", command=self.clear_log).grid(
            row=0, column=6, padx=(0, 8)
        )

        body = ttk.PanedWindow(self, orient="vertical")
        body.grid(row=3, column=0, sticky="nsew", padx=12, pady=(0, 12))

        results_frame = ttk.Frame(body)
        results_frame.columnconfigure(0, weight=1)
        results_frame.rowconfigure(0, weight=1)
        self.results = ttk.Treeview(
            results_frame,
            columns=("result", "detail"),
            show="tree headings",
            height=8,
        )
        self.results.heading("#0", text="Test")
        self.results.heading("result", text="Result")
        self.results.heading("detail", text="Detail")
        self.results.column("#0", width=260, anchor="w")
        self.results.column("result", width=90, anchor="center")
        self.results.column("detail", width=500, anchor="w")
        self.results.grid(row=0, column=0, sticky="nsew")
        results_scroll = ttk.Scrollbar(
            results_frame, orient="vertical", command=self.results.yview
        )
        results_scroll.grid(row=0, column=1, sticky="ns")
        self.results.configure(yscrollcommand=results_scroll.set)

        log_frame = ttk.Frame(body)
        log_frame.columnconfigure(0, weight=1)
        log_frame.rowconfigure(0, weight=1)
        self.log_text = tk.Text(
            log_frame,
            height=18,
            wrap="word",
            state="disabled",
            font=("Consolas", 10),
        )
        self.log_text.grid(row=0, column=0, sticky="nsew")
        log_scroll = ttk.Scrollbar(log_frame, orient="vertical", command=self.log_text.yview)
        log_scroll.grid(row=0, column=1, sticky="ns")
        self.log_text.configure(yscrollcommand=log_scroll.set)

        body.add(results_frame, weight=1)
        body.add(log_frame, weight=3)

    def _attach_log_handler(self) -> None:
        formatter = logging.Formatter("%(asctime)s %(levelname)-8s %(message)s", "%H:%M:%S")
        text_handler = TkTextHandler(self.log_text)
        text_handler.setLevel(logging.DEBUG)
        text_handler.setFormatter(formatter)
        self.logger.addHandler(text_handler)

    def _log_startup(self) -> None:
        self.logger.info("Synchole DLL tester started")
        self.logger.info("Default DLL path: %s", self.dll_path_var.get())
        self.logger.info("Python: %s (%s)", sys.version.split()[0], platform.architecture()[0])
        self.logger.info("Platform: %s", platform.platform())
        self.logger.info("Log file: %s", LOG_FILE)

    def browse_dll(self) -> None:
        selected = filedialog.askopenfilename(
            title="Select synchole_bindings.dll",
            initialdir=str(Path(self.dll_path_var.get()).parent),
            filetypes=(("DLL files", "*.dll"), ("All files", "*.*")),
        )
        if selected:
            self.dll_path_var.set(selected)
            self.logger.info("Selected DLL path: %s", selected)

    def load_dll(self) -> None:
        dll_path = Path(self.dll_path_var.get()).expanduser().resolve()
        try:
            self.client = SyncholeBindings(dll_path=dll_path, logger=self.logger)
            self.client.load()
        except Exception as exc:
            self.client = None
            self.status_var.set("DLL load failed")
            self.logger.exception("DLL load failed: %s", exc)
            messagebox.showerror("DLL load failed", str(exc))
            return

        self.status_var.set(f"Loaded {dll_path.name}")
        self.logger.info("DLL loaded successfully")

    def ensure_client(self) -> SyncholeBindings | None:
        if self.client is None:
            self.load_dll()
        return self.client

    def run_all_tests(self) -> None:
        client = self.ensure_client()
        if client is None:
            return

        self.clear_results()
        self.logger.info("Running all DLL smoke tests")
        results = client.run_smoke_tests()
        for result in results:
            self.results.insert(
                "",
                "end",
                text=result.name,
                values=("PASS" if result.ok else "FAIL", result.detail),
            )
        failed = [result for result in results if not result.ok]
        if failed:
            self.status_var.set(f"{len(failed)} test(s) failed")
            self.logger.error("%d DLL smoke test(s) failed", len(failed))
        else:
            self.status_var.set("All tests passed")
            self.logger.info("All DLL smoke tests passed")

    def call_version(self) -> None:
        client = self.ensure_client()
        if client is None:
            return
        try:
            version = client.version_major()
        except Exception as exc:
            self.status_var.set("version call failed")
            self.logger.exception("synchole_version_major failed: %s", exc)
            return
        self.status_var.set(f"version major = {version}")
        self.logger.info("synchole_version_major returned %s", version)

    def validate_mode(self) -> None:
        client = self.ensure_client()
        if client is None:
            return
        try:
            mode = int(self.mode_var.get())
        except ValueError:
            self.status_var.set("mode must be an integer")
            self.logger.error("Invalid mode input: %s", self.mode_var.get())
            return

        try:
            is_valid = client.storage_mode_is_valid(mode)
        except Exception as exc:
            self.status_var.set("mode validation failed")
            self.logger.exception("synchole_storage_mode_is_valid failed: %s", exc)
            return

        self.status_var.set(f"mode {mode} valid = {is_valid}")
        self.logger.info("synchole_storage_mode_is_valid(%s) returned %s", mode, is_valid)

    def clear_results(self) -> None:
        for item in self.results.get_children():
            self.results.delete(item)

    def clear_log(self) -> None:
        self.log_text.configure(state="normal")
        self.log_text.delete("1.0", "end")
        self.log_text.configure(state="disabled")


def run_smoke_test(dll_path: Path | None = None) -> int:
    logger = configure_logger()
    logger.info("Running command-line smoke test")
    logger.info("Python: %s (%s)", sys.version.split()[0], platform.architecture()[0])

    client = SyncholeBindings(dll_path=dll_path or default_dll_path(), logger=logger)
    try:
        client.load()
        results = client.run_smoke_tests()
    except Exception as exc:
        logger.exception("Smoke test failed before assertions: %s", exc)
        return 1

    failed = [result for result in results if not result.ok]
    for result in results:
        logger.info(
            "%s: %s - %s",
            "PASS" if result.ok else "FAIL",
            result.name,
            result.detail,
        )
    if failed:
        logger.error("%d smoke test(s) failed", len(failed))
        return 1
    logger.info("All smoke tests passed")
    return 0


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Synchole Tkinter DLL tester")
    parser.add_argument("--smoke", action="store_true", help="run smoke tests without the GUI")
    parser.add_argument("--dll", type=Path, help="override the DLL path")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    if args.smoke:
        return run_smoke_test(args.dll)

    app = DllTesterApp(args.dll)
    app.mainloop()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
