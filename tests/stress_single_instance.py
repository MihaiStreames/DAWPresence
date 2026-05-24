#!/usr/bin/env python3
import subprocess
import sys
import time
from pathlib import Path

EXE = Path(__file__).parent.parent / "target" / "release" / "DAWPresence.exe"
SPAWN_COUNT = 30
SPAWN_DELAY = 0.15
HANDLE_POLL_INTERVAL = 2
HANDLE_LEAK_THRESHOLD = 50
SECOND_INSTANCE_MAX_WAIT = 3.0


def get_handle_count(pid: int) -> int | None:
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


def assert_second_instance_exits(exe: Path) -> bool:
    proc = subprocess.Popen([exe], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    try:
        proc.wait(timeout=SECOND_INSTANCE_MAX_WAIT)
        return proc.returncode == 0
    except subprocess.TimeoutExpired:
        proc.kill()
        return False


def main() -> int:
    if not EXE.exists():
        print(f"error: binary not found at {EXE}")
        print("       run `cargo build --release` first")
        return 1

    print(f"launching first instance: {EXE.name}")
    first = subprocess.Popen(
        [EXE], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
    )

    time.sleep(1.0)

    if first.poll() is not None:
        print("error: first instance failed to start")
        return 1

    pid = first.pid
    print(f"first instance pid: {pid}")

    baseline = get_handle_count(pid)
    if baseline is None:
        print("error: could not read handle count")
        first.terminate()
        return 1

    print(f"baseline handle count: {baseline}")
    print(f"\nspawning {SPAWN_COUNT} second instances ({SPAWN_DELAY}s apart)...")

    failures = 0
    for i in range(1, SPAWN_COUNT + 1):
        ok = assert_second_instance_exits(EXE)
        print(f"  [{i:02d}/{SPAWN_COUNT}] {'ok' if ok else 'FAIL'}")
        if not ok:
            failures += 1
        time.sleep(SPAWN_DELAY)

    print(f"\nspawn done. failures: {failures}/{SPAWN_COUNT}")
    print(f"\npolling handle count for {HANDLE_POLL_INTERVAL * 5}s...")

    peak = baseline
    for _ in range(5):
        time.sleep(HANDLE_POLL_INTERVAL)
        count = get_handle_count(pid)
        if count is None:
            print("error: first instance died during handle poll")
            return 1
        peak = max(peak, count)
        print(f"  handle count: {count}")

    handle_growth = peak - baseline
    print(f"\npeak handle growth: {handle_growth} (threshold: {HANDLE_LEAK_THRESHOLD})")

    if first.poll() is not None:
        print("FAIL: first instance is no longer running")
        return 1

    print("first instance still alive: ok")
    first.terminate()
    first.wait(timeout=5)

    print("\n--- results ---")
    print(f"second-instance exit failures : {failures}/{SPAWN_COUNT}")
    print(f"handle growth                 : {handle_growth}")
    print("first instance survived       : yes")

    if failures > 0 or handle_growth > HANDLE_LEAK_THRESHOLD:
        print("status                        : FAIL")
        return 1

    print("status                        : PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())      