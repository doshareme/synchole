from __future__ import annotations

import logging
from dataclasses import dataclass
from time import perf_counter

from synchole_dll import SyncholeDll
from todo_store import TodoStore


@dataclass(frozen=True)
class SyncReport:
    copied_to_left: int
    copied_to_right: int
    conflicts_resolved: int
    duration_ms: int
    dll_version: int


class TodoSyncCoordinator:
    def __init__(
        self,
        left: TodoStore,
        right: TodoStore,
        dll: SyncholeDll,
        logger: logging.Logger | None = None,
    ) -> None:
        self.left = left
        self.right = right
        self.dll = dll
        self.logger = logger or logging.getLogger("synchole_todo_sync")

    def sync_once(self) -> SyncReport:
        started = perf_counter()
        dll_version = self.dll.assert_sql_storage_mode()

        left_rows = self.left.all_rows_by_id()
        right_rows = self.right.all_rows_by_id()
        rows_to_left: list[dict[str, object]] = []
        rows_to_right: list[dict[str, object]] = []
        copied_to_left = 0
        copied_to_right = 0
        conflicts = 0

        for todo_id in sorted(set(left_rows) | set(right_rows)):
            left_row = left_rows.get(todo_id)
            right_row = right_rows.get(todo_id)

            if left_row is not None and right_row is None:
                rows_to_right.append(left_row)
                copied_to_right += 1
                continue

            if right_row is not None and left_row is None:
                rows_to_left.append(right_row)
                copied_to_left += 1
                continue

            if left_row is None or right_row is None:
                continue

            winner = self._pick_winner(left_row, right_row)
            if winner is None:
                continue

            conflicts += 1
            if winner is left_row:
                rows_to_right.append(left_row)
                copied_to_right += 1
            else:
                rows_to_left.append(right_row)
                copied_to_left += 1

        self.left.replace_rows(rows_to_left)
        self.right.replace_rows(rows_to_right)

        elapsed_ms = int((perf_counter() - started) * 1000)
        report = SyncReport(
            copied_to_left=copied_to_left,
            copied_to_right=copied_to_right,
            conflicts_resolved=conflicts,
            duration_ms=elapsed_ms,
            dll_version=dll_version,
        )
        self.logger.debug("Sync report: %s", report)
        return report

    def _pick_winner(self, left_row: dict[str, object], right_row: dict[str, object]) -> dict[str, object] | None:
        if self._same_payload(left_row, right_row):
            return None

        left_stamp = int(left_row["updated_at"])
        right_stamp = int(right_row["updated_at"])
        if left_stamp > right_stamp:
            return left_row
        if right_stamp > left_stamp:
            return right_row

        left_actor = str(left_row["last_modified_by"])
        right_actor = str(right_row["last_modified_by"])
        if left_actor >= right_actor:
            return left_row
        return right_row

    def _same_payload(self, left_row: dict[str, object], right_row: dict[str, object]) -> bool:
        keys = ("title", "completed", "deleted", "updated_at", "last_modified_by")
        return all(left_row[key] == right_row[key] for key in keys)
