from __future__ import annotations

import ctypes
import logging
import os
import platform
from pathlib import Path
from typing import Any


DLL_NAME = "synchole_bindings.dll"
SQL_STORAGE_MODE = 1


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def default_dll_path() -> Path:
    return repo_root() / "target" / "debug" / DLL_NAME


class SyncholeDll:
    def __init__(self, dll_path: Path | None = None, logger: logging.Logger | None = None) -> None:
        self.dll_path = (dll_path or default_dll_path()).expanduser().resolve()
        self.logger = logger or logging.getLogger("synchole_todo_sync")
        self._dll: ctypes.CDLL | None = None
        self._directory_handles: list[object] = []

    def load(self) -> None:
        if self._dll is not None:
            return

        if platform.system() != "Windows":
            self.logger.warning("This sample is intended for Windows DLL loading")

        if not self.dll_path.exists():
            raise FileNotFoundError(
                f"{self.dll_path} does not exist. Build it with: "
                "cargo build -p synchole-bindings --features c-abi"
            )

        self._add_search_directory(self.dll_path.parent)
        deps_dir = self.dll_path.parent / "deps"
        if deps_dir.exists():
            self._add_search_directory(deps_dir)

        self.logger.info("Loading Synchole DLL: %s", self.dll_path)
        self._dll = ctypes.CDLL(str(self.dll_path))
        try:
            self._bind_function("synchole_version_major", [], ctypes.c_uint)
            self._bind_function(
                "synchole_storage_mode_is_valid", [ctypes.c_uint], ctypes.c_bool
            )
        except Exception:
            self._dll = None
            raise
        self.logger.info("Synchole DLL loaded and C ABI functions are available")

    def _add_search_directory(self, directory: Path) -> None:
        if not directory.exists():
            return

        if hasattr(os, "add_dll_directory"):
            self._directory_handles.append(os.add_dll_directory(str(directory)))
            return

        current_path = os.environ.get("PATH", "")
        directory_text = str(directory)
        if directory_text.lower() not in current_path.lower().split(os.pathsep):
            os.environ["PATH"] = directory_text + os.pathsep + current_path

    def _bind_function(self, name: str, argtypes: list[Any], restype: Any) -> None:
        dll = self._require_dll()
        try:
            function = getattr(dll, name)
        except AttributeError as exc:
            raise RuntimeError(
                f"Expected export '{name}' was not found in {self.dll_path}. "
                "Rebuild with: cargo build -p synchole-bindings --features c-abi"
            ) from exc
        function.argtypes = argtypes
        function.restype = restype

    def _require_dll(self) -> ctypes.CDLL:
        if self._dll is None:
            raise RuntimeError("Synchole DLL is not loaded")
        return self._dll

    def version_major(self) -> int:
        self.load()
        value = int(self._require_dll().synchole_version_major())
        self.logger.debug("synchole_version_major() -> %s", value)
        return value

    def storage_mode_is_valid(self, mode: int) -> bool:
        if mode < 0:
            return False
        self.load()
        value = bool(
            self._require_dll().synchole_storage_mode_is_valid(ctypes.c_uint(mode))
        )
        self.logger.debug("synchole_storage_mode_is_valid(%s) -> %s", mode, value)
        return value

    def assert_sql_storage_mode(self) -> int:
        version = self.version_major()
        if not self.storage_mode_is_valid(SQL_STORAGE_MODE):
            raise RuntimeError("Synchole DLL rejected SQL storage mode 1")
        return version
