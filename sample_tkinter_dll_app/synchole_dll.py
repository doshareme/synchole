from __future__ import annotations

import ctypes
import logging
import os
import platform
from dataclasses import dataclass
from pathlib import Path
from typing import Any


DLL_NAME = "synchole_bindings.dll"
STORAGE_MODES = {
    0: "Binary",
    1: "Sql",
    2: "Document",
}


@dataclass(frozen=True)
class TestResult:
    name: str
    ok: bool
    detail: str


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def default_target_debug_dir() -> Path:
    return repo_root() / "target" / "debug"


def default_dll_path() -> Path:
    return default_target_debug_dir() / DLL_NAME


class SyncholeBindings:
    def __init__(self, dll_path: Path | None = None, logger: logging.Logger | None = None) -> None:
        self.dll_path = (dll_path or default_dll_path()).expanduser().resolve()
        self.logger = logger or logging.getLogger("synchole_dll_tester")
        self._dll: ctypes.CDLL | None = None
        self._dll_directory_handles: list[object] = []

    @property
    def is_loaded(self) -> bool:
        return self._dll is not None

    def load(self) -> None:
        if self.is_loaded:
            self.logger.debug("DLL already loaded: %s", self.dll_path)
            return

        if platform.system() != "Windows":
            self.logger.warning("This sample is intended for Windows DLL loading")

        if not self.dll_path.exists():
            raise FileNotFoundError(
                f"{self.dll_path} does not exist. Build it with: "
                "cargo build -p synchole-bindings --features c-abi"
            )

        self._add_dll_search_directory(self.dll_path.parent)
        deps_dir = self.dll_path.parent / "deps"
        if deps_dir.exists():
            self._add_dll_search_directory(deps_dir)

        self.logger.info("Loading DLL: %s", self.dll_path)
        self._dll = ctypes.CDLL(str(self.dll_path))
        try:
            self._bind_functions()
        except Exception:
            self._dll = None
            raise
        self.logger.info("Bound expected exported functions")

    def _add_dll_search_directory(self, directory: Path) -> None:
        if not directory.exists():
            return

        if hasattr(os, "add_dll_directory"):
            self.logger.debug("Adding DLL search directory: %s", directory)
            handle = os.add_dll_directory(str(directory))
            self._dll_directory_handles.append(handle)
            return

        current_path = os.environ.get("PATH", "")
        directory_text = str(directory)
        if directory_text.lower() not in current_path.lower().split(os.pathsep):
            self.logger.debug("Prepending DLL search directory to PATH: %s", directory)
            os.environ["PATH"] = directory_text + os.pathsep + current_path

    def _bind_functions(self) -> None:
        self._bind_function("synchole_version_major", [], ctypes.c_uint)
        self._bind_function(
            "synchole_storage_mode_is_valid", [ctypes.c_uint], ctypes.c_bool
        )

    def _bind_function(
        self,
        name: str,
        argtypes: list[Any],
        restype: Any,
    ) -> None:
        dll = self._require_dll()
        try:
            function = getattr(dll, name)
        except AttributeError as exc:
            raise RuntimeError(
                f"Expected export '{name}' was not found in {self.dll_path}. "
                "Rebuild the debug DLL with: "
                "cargo build -p synchole-bindings --features c-abi"
            ) from exc
        function.argtypes = argtypes
        function.restype = restype

    def _require_dll(self) -> ctypes.CDLL:
        if self._dll is None:
            raise RuntimeError("DLL is not loaded")
        return self._dll

    def version_major(self) -> int:
        dll = self._require_dll()
        value = int(dll.synchole_version_major())
        self.logger.debug("synchole_version_major() -> %s", value)
        return value

    def storage_mode_is_valid(self, mode: int) -> bool:
        if mode < 0:
            raise ValueError("mode must be unsigned")
        dll = self._require_dll()
        result = bool(dll.synchole_storage_mode_is_valid(ctypes.c_uint(mode)))
        self.logger.debug("synchole_storage_mode_is_valid(%s) -> %s", mode, result)
        return result

    def run_smoke_tests(self) -> list[TestResult]:
        self.load()

        results: list[TestResult] = []

        version = self.version_major()
        results.append(
            TestResult(
                name="synchole_version_major",
                ok=version == 0,
                detail=f"returned {version}, expected 0 for current debug C ABI",
            )
        )

        for mode, name in STORAGE_MODES.items():
            is_valid = self.storage_mode_is_valid(mode)
            results.append(
                TestResult(
                    name=f"storage mode {mode} ({name})",
                    ok=is_valid is True,
                    detail=f"returned {is_valid}, expected True",
                )
            )

        for mode in (3, 999):
            is_valid = self.storage_mode_is_valid(mode)
            results.append(
                TestResult(
                    name=f"storage mode {mode} (invalid)",
                    ok=is_valid is False,
                    detail=f"returned {is_valid}, expected False",
                )
            )

        return results
