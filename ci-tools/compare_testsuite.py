#!/usr/bin/env python3

import json
import sys
import os.path


def sanitize_filepath(path: str) -> str:
    if path.startswith("./tests/specification/testsuite/"):
        path = path[len("./tests/specification/testsuite/") :]
    return path


def sanatize_table_item(item: str) -> str:
    if item is None:
        return "-"
    new = ""
    new = item.replace("`", "'")
    new = new.replace("|", "/")
    new = new.replace("\n", " ")

    return new


def print_missing(original: bool, new: bool):
    if not original and new:
        print(
            "Uh-oh! It looks like the original testsuite results do not exist! "
            "This is likely due to the testsuite not being updated on the target branch. "
            "This PR will fix this issue. No comparison possible. "
        )
    elif original and not new:
        print(
            "Uh-oh! It looks like this PR couldn't generate the testsuite results. "
            "Please make sure this isn't indicative of a bigger error! "
        )
    elif not original and not new:
        print(
            "Uh-oh! It looks like the original testsuite results and the new results are both missing! "
            "This is certainly indicative of an error in either the branches or this GH action! "
        )


def get_summary(entries) -> str:
    """
    Logic:
    - For each entry
        - If script error, print error
        - If Assert:
            - Count number of passing and failing asserts, calculate percentage
    """
    result = ""
    result += "<details><summary>Click here to open</summary>\n\n"
    result += "| **File** | **Passed Asserts** | **Failed Asserts** | **% Passed** | **Notes** |\n"
    result += "|:--------:|:------------------:|:------------------:|:------------:|-----------|\n"

    for entry in entries:
        file = sanitize_filepath(entry["filepath"])
        result += f"| {file}"
        if "Assert" in entry["data"]:
            failed = 0
            passed = 0
            for an_assert in entry["data"]["Assert"]["results"]:
                if an_assert["error"] is not None:
                    failed += 1
                else:
                    passed += 1

            total = failed + passed
            if total != 0:
                percent = round(passed / total * 100, 2)
            else:
                percent = 0
            result += f"| {passed} / {total} | {failed} / {total} | {percent}% | - |\n"
        else:  # "ScriptError"
            script_error = entry["data"]["ScriptError"]
            result += f"| - | - | - |"
            # result += f'Error: `{sanatize_table_item(script_error["error"])}`<br>'
            result += f'Context: `{sanatize_table_item(script_error["context"])}`<br>'
            result += f'Line: `{script_error["line_number"]}`<br>'
            # result += f'Command: `{sanatize_table_item(script_error["command"])}`'
            result += "|\n"

    result += "\n\n</details>\n"
    return result


def get_delta(old_entries, new_entries) -> str:
    """
    Delta cases:
    1. File deleted
        - in old entries (?)
        - in new entries (?)
    2. ScriptError
        - in old but not in new (good)
        - in new but not in old (bad)
    3. Asserts
        - modified file/asserts (?)
        - Error status:
            i. Failing in old, Passing in new (good)
            ii. Failing in new, Passing in old (bad)
            iii. Some failing and some passing in new (good and bad = ?)
                - (Basically a combination of case [i] and case [ii])
    """
    result = ""

    def find_entry(haystack, filepath):
        return next(
            (entry for entry in haystack if entry["filepath"] == filepath),
            None
        )

    def find_assert(haystack, line, command):
        return next(
            (an_assert for an_assert in haystack if an_assert["line_number"] == line and an_assert["command"] == command),
            None
        )

    # Get reunion of entries
    all_entries = list(old_entries)
    for entry in new_entries:
        if find_entry(old_entries, entry["filepath"]) is None:
            all_entries.append(entry)

    for entry in all_entries:
        full_file = entry["filepath"]
        file = sanitize_filepath(entry["filepath"])

        old_entry = find_entry(old_entries, full_file)
        new_entry = find_entry(new_entries, full_file)

        if new_entry is None:
            result += f"| {file} | File missing in this PR | ⚠️ |\n"
            continue
        elif old_entry is None:
            result += f"| {file} | File missing in target branch | ⚠️ |\n"
            continue

        # First, compare if script error
        se_old = "ScriptError" in old_entry["data"]
        se_new = "ScriptError" in new_entry["data"]

        if se_old and se_new: # Script error both in old and new
            continue


        if se_old and not se_new:
            result += f"| {file} | File now compiles | ✅ |\n"
        elif not se_old and se_new:
            result += f"| {file} | File no longer compiles | ❌ |\n"
        elif not se_old and not se_new:
            # Secondly, test if file is the same.
            asserts_old = old_entry["data"]["Assert"]["results"]
            asserts_new = new_entry["data"]["Assert"]["results"]
            same_file_contents = len(asserts_old) == len(asserts_new) and all(
                [
                    find_assert(asserts_new, old["line_number"], old["command"]) is not None
                    for old in asserts_old
                ]
            )

            if not same_file_contents:
                result += f"| {file} | File has changed. Cannot check | ⚠️ |\n"
            else:
                # Sort by line number
                asserts_old = sorted(
                    asserts_old, key=lambda an_assert: an_assert["line_number"]
                )
                asserts_new = sorted(
                    asserts_new, key=lambda an_assert: an_assert["line_number"]
                )

                new_passing = 0
                new_failing = 0
                for i in range(len(asserts_old)):
                    old_is_err = asserts_old[i]["error"] is not None
                    new_is_err = asserts_new[i]["error"] is not None

                    if old_is_err and not new_is_err:
                        new_passing += 1
                    elif not old_is_err and new_is_err:
                        new_failing += 1

                if new_passing != 0 or new_failing != 0:
                    result += f"| {file} | "
                    if new_passing != 0 and new_failing == 0:
                        result += f"+{new_passing} asserts PASS | ✅ |\n"
                    elif new_passing == 0 and new_failing != 0:
                        result += f"-{new_failing} asserts FAIL | ❌ |\n"
                    else:
                        result += f"+{new_passing} asserts PASS<br>-{new_failing} asserts FAIL | ⚠️ |\n"

    if result != "":
        header = ""
        header += "| **File** | **Notes** | ❓ |\n"
        header += "|:--------:|:---------:|:--:|\n"
        return header + result
    else:
        return "<b> No changes detected. </b>"


def main():
    if len(sys.argv) != 3:
        print("Usage: script.py <original_json> <new_json>")
        sys.exit(1)

    original_path = sys.argv[1]
    new_path = sys.argv[2]

    print("# 🗒️ Testsuite Report\n\n")

    original_exists = os.path.isfile(original_path)
    new_exists = os.path.isfile(new_path)
    if not original_exists or not new_exists:
        print_missing(original_exists, new_exists)
        return

    old_data = None
    with open(original_path, "r") as f:
        old_data = json.load(f)

    new_data = None
    with open(new_path, "r") as f:
        new_data = json.load(f)

    if original_exists and new_exists:
        print("## PR delta: \n")
        print(get_delta(old_data["entries"], new_data["entries"]))
        print("\n")

    # print("## Target summary: \n")
    # print(get_overview(old_data["entries"]))
    # print("\n")

    if new_exists:
        print("## PR summary: \n")
        print(get_summary(new_data["entries"]))
        print("\n")


if __name__ == "__main__":
    main()
