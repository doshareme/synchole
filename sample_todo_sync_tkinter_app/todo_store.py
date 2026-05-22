from __future__ import annotations

import logging
import sqlite3
import time
import uuid
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class TodoItem:
    id: str
    title: str
    completed: bool
    deleted: bool
    created_at: int
    updated_at: int
    origin: str
    last_modified_by: str

    @property
    def updated_at_text(self) -> str:
        return datetime.fromtimestamp(self.updated_at / 1000).strftime("%H:%M:%S")


def now_ms() -> int:
    return time.time_ns() // 1_000_000


class TodoStore:
    def __init__(self, path: Path, device_id: str, logger: logging.Logger | None = None) -> None:
        self.path = path
        self.device_id = device_id
        self.logger = logger or logging.getLogger("synchole_todo_sync")
        self._connection: sqlite3.Connection | None = None
        self._last_timestamp = 0

    def initialize(self) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        connection = sqlite3.connect(str(self.path))
        connection.row_factory = sqlite3.Row
        connection.execute("PRAGMA journal_mode=WAL")
        connection.execute("PRAGMA synchronous=NORMAL")
        connection.execute(
            """
            CREATE TABLE IF NOT EXISTS todos (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                deleted INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                origin TEXT NOT NULL,
                last_modified_by TEXT NOT NULL
            )
            """
        )
        connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_todos_deleted_updated "
            "ON todos(deleted, updated_at)"
        )
        connection.commit()
        self._connection = connection
        row = connection.execute("SELECT MAX(updated_at) AS updated_at FROM todos").fetchone()
        if row is not None and row["updated_at"] is not None:
            self._last_timestamp = int(row["updated_at"])
        self.logger.info("%s initialized SQLite DB: %s", self.device_id, self.path)

    def close(self) -> None:
        if self._connection is not None:
            self._connection.close()
            self._connection = None

    def _conn(self) -> sqlite3.Connection:
        if self._connection is None:
            raise RuntimeError(f"{self.device_id} store is not initialized")
        return self._connection

    def add(self, title: str) -> TodoItem:
        timestamp = self._next_timestamp()
        item = TodoItem(
            id=str(uuid.uuid4()),
            title=title,
            completed=False,
            deleted=False,
            created_at=timestamp,
            updated_at=timestamp,
            origin=self.device_id,
            last_modified_by=self.device_id,
        )
        self._conn().execute(
            """
            INSERT INTO todos (
                id, title, completed, deleted, created_at, updated_at, origin,
                last_modified_by
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            self._item_values(item),
        )
        self._conn().commit()
        return item

    def bulk_add(self, count: int, title_prefix: str) -> int:
        if count < 1 or count > 10_000:
            raise ValueError("count must be between 1 and 10000")

        rows = []
        for index in range(1, count + 1):
            timestamp = self._next_timestamp()
            rows.append(
                (
                    str(uuid.uuid4()),
                    f"{title_prefix} {index:05d}",
                    0,
                    0,
                    timestamp,
                    timestamp,
                    self.device_id,
                    self.device_id,
                )
            )

        self._conn().executemany(
            """
            INSERT INTO todos (
                id, title, completed, deleted, created_at, updated_at, origin,
                last_modified_by
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            rows,
        )
        self._conn().commit()
        return count

    def get(self, todo_id: str) -> TodoItem:
        row = self._conn().execute(
            "SELECT * FROM todos WHERE id = ?", (todo_id,)
        ).fetchone()
        if row is None:
            raise KeyError(todo_id)
        return self._row_to_item(row)

    def list_active(self) -> list[TodoItem]:
        rows = self._conn().execute(
            """
            SELECT * FROM todos
            WHERE deleted = 0
            ORDER BY completed ASC, updated_at DESC, title COLLATE NOCASE ASC
            """
        ).fetchall()
        return [self._row_to_item(row) for row in rows]

    def count_active(self) -> int:
        row = self._conn().execute(
            "SELECT COUNT(*) AS total FROM todos WHERE deleted = 0"
        ).fetchone()
        return int(row["total"])

    def set_completed(self, todo_id: str, completed: bool) -> None:
        self._update_fields(todo_id, completed=1 if completed else 0)

    def rename(self, todo_id: str, title: str) -> None:
        self._update_fields(todo_id, title=title)

    def delete(self, todo_id: str) -> None:
        self._update_fields(todo_id, deleted=1)

    def _update_fields(self, todo_id: str, **fields: Any) -> None:
        fields["updated_at"] = self._next_timestamp()
        fields["last_modified_by"] = self.device_id
        assignments = ", ".join(f"{key} = ?" for key in fields)
        values = list(fields.values())
        values.append(todo_id)
        self._conn().execute(
            f"UPDATE todos SET {assignments} WHERE id = ?",
            values,
        )
        self._conn().commit()

    def _next_timestamp(self) -> int:
        timestamp = max(now_ms(), self._last_timestamp + 1)
        self._last_timestamp = timestamp
        return timestamp

    def all_rows_by_id(self) -> dict[str, dict[str, object]]:
        rows = self._conn().execute("SELECT * FROM todos").fetchall()
        return {str(row["id"]): self._row_to_dict(row) for row in rows}

    def replace_row(self, row: dict[str, object]) -> None:
        self.replace_rows([row])

    def replace_rows(self, rows: list[dict[str, object]]) -> None:
        if not rows:
            return

        self._conn().executemany(
            """
            INSERT INTO todos (
                id, title, completed, deleted, created_at, updated_at, origin,
                last_modified_by
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                completed = excluded.completed,
                deleted = excluded.deleted,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at,
                origin = excluded.origin,
                last_modified_by = excluded.last_modified_by
            """,
            [self._row_values(row) for row in rows],
        )
        self._conn().commit()
        self._last_timestamp = max(
            self._last_timestamp,
            max(int(row["updated_at"]) for row in rows),
        )

    def _row_values(self, row: dict[str, object]) -> tuple[object, ...]:
        return (
            row["id"],
            row["title"],
            row["completed"],
            row["deleted"],
            row["created_at"],
            row["updated_at"],
            row["origin"],
            row["last_modified_by"],
        )

    def _row_to_item(self, row: sqlite3.Row) -> TodoItem:
        return TodoItem(
            id=str(row["id"]),
            title=str(row["title"]),
            completed=bool(row["completed"]),
            deleted=bool(row["deleted"]),
            created_at=int(row["created_at"]),
            updated_at=int(row["updated_at"]),
            origin=str(row["origin"]),
            last_modified_by=str(row["last_modified_by"]),
        )

    def _row_to_dict(self, row: sqlite3.Row) -> dict[str, object]:
        return {
            "id": str(row["id"]),
            "title": str(row["title"]),
            "completed": int(row["completed"]),
            "deleted": int(row["deleted"]),
            "created_at": int(row["created_at"]),
            "updated_at": int(row["updated_at"]),
            "origin": str(row["origin"]),
            "last_modified_by": str(row["last_modified_by"]),
        }

    def _item_values(self, item: TodoItem) -> tuple[object, ...]:
        return (
            item.id,
            item.title,
            1 if item.completed else 0,
            1 if item.deleted else 0,
            item.created_at,
            item.updated_at,
            item.origin,
            item.last_modified_by,
        )
