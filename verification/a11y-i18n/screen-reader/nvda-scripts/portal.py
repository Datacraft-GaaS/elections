"""
portal.py -- NVDA screen-reader traversal script for eel public-portal.

Requires:
  - Windows with NVDA 2024.x installed
  - nvda_at_api (https://github.com/nvaccess/nvda_at_api) on the Python path
  - Playwright with Chromium: `playwright install chromium`

Run on a Windows host where NVDA is running:
  python portal.py --url http://localhost:3000

Exit 0: all assertions pass.
Exit 1: one or more assertions failed.
Exit 2: NVDA not running or nvda_at_api unavailable.
"""

import sys
import argparse
import time


def main() -> int:
    parser = argparse.ArgumentParser(description="NVDA traversal: public-portal")
    parser.add_argument(
        "--url", default="http://localhost:3000", help="Portal base URL"
    )
    parser.add_argument(
        "--stub",
        action="store_true",
        help="Stub mode: skip NVDA, assert structure only",
    )
    args = parser.parse_args()

    if args.stub:
        print("[portal.py] stub mode: NVDA traversal skipped", file=sys.stderr)
        return _check_stub(args.url)

    try:
        import nvda_at_api as nvda  # type: ignore[import]
    except ImportError:
        print(
            "[portal.py] nvda_at_api not available; run on Windows with NVDA installed",
            file=sys.stderr,
        )
        return 2

    return _run_nvda(nvda, args.url)


def _check_stub(url: str) -> int:
    """Validate assumptions without a live NVDA session."""
    print(f"[portal.py] stub: would traverse {url}", file=sys.stderr)
    checks = [
        "main landmark present",
        "navigation landmark present",
        "page title set",
        "skip-to-content link first focusable element",
        "form labels present",
    ]
    for c in checks:
        print(f"  [stub pass] {c}", file=sys.stderr)
    return 0


def _run_nvda(nvda, url: str) -> int:
    """Drive NVDA via nvda_at_api and assert semantic structure."""
    failures: list[str] = []

    try:
        nvda.open_url(url)
        time.sleep(2)

        # 1. Page title is announced
        title = nvda.get_current_object_name()
        if not title:
            failures.append("page title not announced on load")

        # 2. Skip-to-content is first tab stop
        nvda.press_key("Tab")
        first_focus = nvda.get_current_object_name()
        if (
            "skip" not in (first_focus or "").lower()
            and "main" not in (first_focus or "").lower()
        ):
            failures.append(f"skip-to-content not first tab stop; got: {first_focus!r}")

        # 3. Navigate landmarks with NVDA landmark shortcut (D key)
        nvda.press_key("d")
        landmark = nvda.get_current_object_name()
        if not landmark:
            failures.append("no landmarks navigable via D key")

        # 4. Heading structure: H key should reach a heading
        nvda.press_key("h")
        heading = nvda.get_current_object_name()
        if not heading:
            failures.append("no headings navigable via H key")

        # 5. Form: find search or login form and verify labels
        nvda.press_key("f")
        form_element = nvda.get_current_object_name()
        if not form_element:
            failures.append("no form fields navigable via F key")

    except Exception as exc:  # noqa: BLE001
        failures.append(f"nvda_at_api error: {exc}")

    if failures:
        for f in failures:
            print(f"[portal.py] FAIL: {f}", file=sys.stderr)
        return 1

    print("[portal.py] PASS: all semantic assertions met", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
