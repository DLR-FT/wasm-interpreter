#!/usr/bin/env python3

import json
import sys
from enum import Enum
from dataclasses import dataclass
from typing import Optional


class Progress(Enum):
    REGRESSED = "regressed"
    SAME = "same"
    PROGRESSED = "progressed"


@dataclass
class TestEntry:
    filename: str
    compiled: bool
    total: int
    passed: int
    failed: int

    @property
    def pass_percentage(self) -> float:
        if self.total == 0:
            return 0.0

        return round(self.passed / self.total * 100, 2)


@dataclass
class ComparisonResult:
    filename: str
    compiled_original: Optional[bool]
    compiled_new: Optional[bool]
    percentage_original: Optional[float]
    percentage_new: Optional[float]

    def progress(self) -> Progress:
        if self.compiled_original is None or self.percentage_original is None:
            return Progress.PROGRESSED
        if self.compiled_new is None or self.percentage_new is None:
            return Progress.REGRESSED

        if self.compiled_original and not self.compiled_new:
            return Progress.REGRESSED
        if not self.compiled_original and self.compiled_new:
            return Progress.PROGRESSED

        if self.percentage_new == self.percentage_original:
            return Progress.SAME
        elif self.percentage_new > self.percentage_original:
            return Progress.PROGRESSED
        else:
            return Progress.REGRESSED


def load_and_parse_json(filepath: str) -> dict[str, TestEntry]:
    with open(filepath, "r") as f:
        data = json.load(f)

    result: dict[str, TestEntry] = {}
    for entry in data.get("entries", []):
        filename = entry["filename"]
        # Strip './tests/specification/testsuite/' prefix if present
        if filename.startswith("./tests/specification/testsuite/"):
            filename = filename[len("./tests/specification/testsuite/") :]

        result[filename] = TestEntry(
            filename=filename,
            compiled=entry["compiled"],
            total=entry["tests_total"],
            passed=entry["tests_passed"],
            failed=entry["tests_failed"],
        )

    return result


def compare_test_results(original_path: str, new_path: str) -> list[ComparisonResult]:
    original_entries = load_and_parse_json(original_path)
    new_entries = load_and_parse_json(new_path)

    all_files = set(original_entries.keys()) | set(new_entries.keys())

    results: list[ComparisonResult] = []
    for filename in sorted(all_files):
        original = original_entries.get(filename)
        new = new_entries.get(filename)

        result = ComparisonResult(
            filename=filename,
            compiled_original=original.compiled if original else None,
            compiled_new=new.compiled if new else None,
            percentage_original=original.pass_percentage if original else None,
            percentage_new=new.pass_percentage if new else None,
        )
        results.append(result)

    return results


def main():
    if len(sys.argv) != 3:
        print("Usage: script.py <original_json> <new_json>")
        sys.exit(1)

    original_path = sys.argv[1]
    new_path = sys.argv[2]

    results = compare_test_results(original_path, new_path)

    markdown_table = ""
    entries_modified = False

    print("# 🗒️ Testsuite Report\n\n")
    markdown_table += "| File | Compilation (Old → New) | Pass Rate (Old → New) | Status |\n"
    markdown_table += "|------|-------------------------|-----------------------|--------|\n"

    for result in results:
        compiled_original = (
            "✅"
            if result.compiled_original
            else "❌" if result.compiled_original is not None else "❓"
        )
        compiled_new = (
            "✅" if result.compiled_new else "❌" if result.compiled_new is not None else "❓"
        )
        compilation = f"{compiled_original} → {compiled_new}"

        percentage_original = (
            result.percentage_original if result.percentage_original is not None else "❓"
        )
        percentage_new = result.percentage_new if result.percentage_new is not None else "❓"
        percentages = f"{percentage_original}% → {percentage_new}%"

        status = result.progress()
        status_str = (
            "➖"
            if status == Progress.SAME
            else "✅ Progressed" if status == Progress.PROGRESSED else "❌ Regressed"
        )

        # There are a LOT of entries, let's skip the ones that are the same
        if status != Progress.SAME:
            entries_modified = True
            markdown_table += (
                f"| {result.filename} | {compilation} | {percentages} | {status_str} |\n"
            )

    if not entries_modified:
        print("No changes detected.")
    else:
        print(markdown_table)


if __name__ == "__main__":
    main()
