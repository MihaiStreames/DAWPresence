#!/usr/bin/env python3

from pathlib import Path

EXE: Path = Path(__file__).parent.parent / "target" / "release" / "DAWPresence.exe"
BAR_WIDTH: int = 20


def progress_bar(current: int | float, total: int | float) -> str:
    filled = int(BAR_WIDTH * min(current / total, 1.0)) if total > 0 else BAR_WIDTH
    bar = "█" * filled + "░" * (BAR_WIDTH - filled)
    return f"[{bar}] {int(current)}/{int(total)}"
