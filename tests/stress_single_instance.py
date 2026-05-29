#!/usr/bin/env python3

import subprocess
import sys
import time
from dataclasses import dataclass

from utils import EXE, progress_bar

SPAWN_COUNT: int = 30
SPAWN_DELAY: float = 0.15
HANDLE_POLL_INTERVAL: int = 2
HANDLE_POLL_ROUNDS: int = 30
HANDLE_LEAK_THRESHOLD: int = 50
SECOND_INSTANCE_MAX_WAIT: float = 3.0


@dataclass(frozen=True)
class StressResult:
    failures: int
    handle_growth: int


def _get_handle_count(pid: int) -> int | None:
    result = subprocess.run(
        [
            "powershell",
            "-NoProfile",
            "-Command",
            f"(Get-Process -Id {pid}).HandleCount",
        ],
        capture_output=True,
        text=True,
    )

    try:
        return int(result.stdout.strip())
    except ValueError:
        return None


def second_instance_exits(exe=EXE) -> bool:
    proc = subprocess.Popen([exe], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    try:
        proc.wait(timeout=SECOND_INSTANCE_MAX_WAIT)
        return proc.returncode == 0
    except subprocess.TimeoutExpired:
        proc.kill()
        return False


def _run_spawn_stress() -> int:
    failures = 0

    for i in range(1, SPAWN_COUNT + 1):
        if not second_instance_exits():
            failures += 1

        print(
            f"\r{progress_bar(i, SPAWN_COUNT)}  failures={failures}", end="", flush=True
        )

        time.sleep(SPAWN_DELAY)

    print()
    return failures


def _poll_handle_peak(pid: int, baseline: int) -> int | None:
    peak = baseline

    for i in range(1, HANDLE_POLL_ROUNDS + 1):
        time.sleep(HANDLE_POLL_INTERVAL)

        count = _get_handle_count(pid)
        if count is None:
            return None

        peak = max(peak, count)
        print(
            f"\r{progress_bar(i, HANDLE_POLL_ROUNDS)}  peak={peak}", end="", flush=True
        )

    print()
    return peak


def _run_stress(pid: int, baseline: int) -> StressResult | None:
    failures = _run_spawn_stress()

    peak = _poll_handle_peak(pid, baseline)
    if peak is None:
        return None

    return StressResult(failures=failures, handle_growth=peak - baseline)


def main() -> int:
    if not EXE.exists():
        print(f"Error: binary not found at {EXE}")
        return 1

    first = subprocess.Popen(
        [EXE], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
    )

    time.sleep(1.0)

    baseline = _get_handle_count(first.pid)
    if baseline is None:
        first.terminate()
        return 1

    result = _run_stress(first.pid, baseline)
    first.terminate()

    try:
        first.wait(timeout=5)
    except subprocess.TimeoutExpired:
        first.kill()
        first.wait()

    if result is None:
        print("Error: process died during handle poll")
        return 1

    print(
        f"failures: {result.failures}/{SPAWN_COUNT}  handle growth: {result.handle_growth}"
    )
    return (
        0
        if result.failures == 0 and result.handle_growth <= HANDLE_LEAK_THRESHOLD
        else 1
    )


if __name__ == "__main__":
    sys.exit(main())
