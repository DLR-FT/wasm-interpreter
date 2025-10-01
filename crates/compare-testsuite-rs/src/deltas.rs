use std::{borrow::Cow, fmt::Display};

use anyhow::{bail, ensure};
use itertools::Itertools;

use crate::{
    ci_reports::{
        CIAssert, CIFullReport,
        CIReportData::{self, Assert, ScriptError},
    },
    sanitize_path, write_details_summary,
};

pub fn generate<'a>(
    old_report: &'a CIFullReport,
    new_report: &'a CIFullReport,
) -> anyhow::Result<FileDeltas<'a>> {
    let all_deduplicated_filepaths = old_report
        .entries
        .iter()
        .chain(new_report.entries.iter())
        .map(|entry| &entry.filepath)
        .unique();

    let filepath_old_new = all_deduplicated_filepaths.map(|filepath| {
        (
            filepath,
            old_report
                .entries
                .iter()
                .find_map(|entry| (&entry.filepath == filepath).then_some(&entry.data)),
            new_report
                .entries
                .iter()
                .find_map(|entry| (&entry.filepath == filepath).then_some(&entry.data)),
        )
    });

    let mut deltas = filepath_old_new
        .flat_map(|(filepath, old_entry, new_entry)| {
            generate_file_delta_type(filepath, old_entry, new_entry)
                .transpose()
                .map(|ty| ty.map(|ty| FileDelta { filepath, ty }))
        })
        .collect::<anyhow::Result<Vec<FileDelta>>>()?;

    // We want to sort deltas by their icons and alphabetically
    deltas.sort_by(|a, b| match a.ty.to_icon().cmp(&b.ty.to_icon()) {
        std::cmp::Ordering::Equal => a.filepath.cmp(b.filepath),
        other => other,
    });

    Ok(FileDeltas(deltas))
}

fn generate_file_delta_type<'r>(
    filepath: &'r str,
    old: Option<&'r CIReportData>,
    new: Option<&'r CIReportData>,
) -> anyhow::Result<Option<FileDeltaType<'r>>> {
    let ty = match (old, new) {
        (None, None) => unreachable!("filepath must exist in at least one report"),
        (Some(_old), None) => bail!("results for {filepath} missing for new version"),
        (None, Some(_new)) => bail!("results for {filepath} missing for old version"),
        (
            Some(ScriptError {
                error: old_error,
                context: old_context,
                line_number: old_line_number,
                ..
            }),
            Some(ScriptError {
                error: new_error,
                context: new_context,
                line_number: new_line_number,
                ..
            }),
        ) => (old_error != new_error).then_some(FileDeltaType::ScriptErrorChanged {
            old_error,
            old_context,
            old_line_number: *old_line_number,
            new_error,
            new_context,
            new_line_number: *new_line_number,
        }),
        (
            Some(Assert { results }),
            Some(ScriptError {
                error,
                context,
                line_number,
                ..
            }),
        ) => Some(FileDeltaType::NewScriptError {
            old_num_passing_asserts: results
                .iter()
                .filter(|assert| assert.error.is_none())
                .count(),
            old_num_asserts: results.len(),
            new_error: error,
            new_context: context,
            new_line_number: *line_number,
        }),
        (
            Some(ScriptError {
                error,
                context,
                line_number,
                _command: _,
            }),
            Some(Assert { results }),
        ) => Some(FileDeltaType::ScriptErrorResolved {
            old_error: error,
            old_context: context,
            old_line_number: *line_number,
            new_num_passing_asserts: results
                .iter()
                .filter(|assert| assert.error.is_none())
                .count(),
            new_num_asserts: results.len(),
        }),
        (Some(Assert { results: old }), Some(Assert { results: new })) => {
            generate_file_delta_assert(filepath, old, new)?
        }
    };

    Ok(ty)
}

fn generate_file_delta_assert<'r>(
    filepath: &'r str,
    old: &'r [CIAssert],
    new: &'r [CIAssert],
) -> anyhow::Result<Option<FileDeltaType<'r>>> {
    struct OldNewAssertPair<'report> {
        line_number: u32,
        command: &'report str,
        old_error: Option<&'report String>,
        new_error: Option<&'report String>,
    }

    let pairs = itertools::iproduct!(old.iter(), new.iter())
        .filter_map(|(old, new)| {
            (old.line_number == new.line_number && old.command == new.command).then(|| {
                OldNewAssertPair {
                    line_number: old.line_number,
                    command: &old.command,
                    old_error: old.error.as_ref(),
                    new_error: new.error.as_ref(),
                }
            })
        })
        .collect_vec();
    ensure!(pairs.len() == old.len() && pairs.len() == new.len(), "reports for {filepath} contains different asserts. make sure that the same testsuite is used for both runs");

    let mut now_passing = Vec::new();
    let mut now_failing = Vec::new();
    let mut now_with_different_error_message = Vec::new();

    for assert_pair in &pairs {
        match assert_pair {
            OldNewAssertPair {
                line_number,
                command,
                old_error: Some(old_error),
                new_error: None,
            } => now_passing.push(NowPassingAssert {
                line_number: *line_number,
                _command: command,
                previous_error: old_error,
            }),
            OldNewAssertPair {
                line_number,
                command,
                old_error: None,
                new_error: Some(new_error),
            } => {
                now_failing.push(NowFailingAssert {
                    line_number: *line_number,
                    _command: command,
                    new_error,
                });
            }
            OldNewAssertPair {
                line_number,
                command,
                old_error: Some(old_error),
                new_error: Some(new_error),
            } if old_error != new_error => {
                now_with_different_error_message.push(NowDifferentErrorAssert {
                    line_number: *line_number,
                    _command: command,
                    old_error,
                    new_error,
                })
            }
            // All other cases do not produce differences
            _ => {}
        }
    }

    match (
        now_passing.is_empty(),
        now_failing.is_empty(),
        now_with_different_error_message.is_empty(),
    ) {
        (_, _, false) | (false, false, true) => Ok(Some(FileDeltaType::AssertsChanged {
            now_passing,
            now_failing,
            now_different_error: now_with_different_error_message,
        })),
        (true, false, true) => Ok(Some(FileDeltaType::AssertsFailing {
            now_failing,
            num_asserts: pairs.len(),
            old_num_passing_asserts: old.iter().filter(|assert| assert.error.is_none()).count(),
        })),
        (false, true, true) => Ok(Some(FileDeltaType::AssertsPassing {
            now_passing,
            num_asserts: pairs.len(),
            old_num_passing_asserts: old.iter().filter(|assert| assert.error.is_none()).count(),
        })),
        (true, true, true) => Ok(None),
    }
}

pub struct FileDeltas<'report>(Vec<FileDelta<'report>>);
impl Display for FileDeltas<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return writeln!(f, "<b> No changes detected </b>");
        }

        write!(
            f,
            "\
## Changes

|    | **File** | **Diff** | **Now passing** | **Now passing%** |
|:--:|:--------:|----------|-----------------|------------------|
"
        )?;
        self.0.iter().try_for_each(|delta| writeln!(f, "{delta}"))
    }
}

struct FileDelta<'report> {
    filepath: &'report str,
    ty: FileDeltaType<'report>,
}

enum FileDeltaType<'report> {
    NewScriptError {
        old_num_passing_asserts: usize,
        old_num_asserts: usize,
        new_error: &'report str,
        new_context: &'report str,
        new_line_number: Option<u32>,
    },
    ScriptErrorResolved {
        old_error: &'report str,
        old_context: &'report str,
        old_line_number: Option<u32>,

        new_num_passing_asserts: usize,
        new_num_asserts: usize,
    },
    ScriptErrorChanged {
        old_error: &'report str,
        old_context: &'report str,
        old_line_number: Option<u32>,
        new_error: &'report str,
        new_context: &'report str,
        new_line_number: Option<u32>,
    },
    AssertsPassing {
        num_asserts: usize,
        old_num_passing_asserts: usize,
        now_passing: Vec<NowPassingAssert<'report>>,
    },
    AssertsFailing {
        num_asserts: usize,
        old_num_passing_asserts: usize,
        now_failing: Vec<NowFailingAssert<'report>>,
    },
    AssertsChanged {
        now_passing: Vec<NowPassingAssert<'report>>,
        now_failing: Vec<NowFailingAssert<'report>>,
        now_different_error: Vec<NowDifferentErrorAssert<'report>>,
    },
}

impl FileDeltaType<'_> {
    fn to_icon(&self) -> FileDeltaTypeIcon {
        match self {
            FileDeltaType::NewScriptError { .. } | FileDeltaType::AssertsFailing { .. } => {
                FileDeltaTypeIcon::Bad
            }
            FileDeltaType::ScriptErrorResolved { .. } | FileDeltaType::AssertsPassing { .. } => {
                FileDeltaTypeIcon::Good
            }
            FileDeltaType::ScriptErrorChanged { .. } | FileDeltaType::AssertsChanged { .. } => {
                FileDeltaTypeIcon::Warn
            }
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum FileDeltaTypeIcon {
    Bad = 1,
    Warn = 2,
    Good = 3,
}

impl Display for FileDeltaTypeIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileDeltaTypeIcon::Bad => write!(f, ":x:"),
            FileDeltaTypeIcon::Warn => write!(f, ":warning:"),
            FileDeltaTypeIcon::Good => write!(f, ":white_check_mark:"),
        }
    }
}

struct NowPassingAssert<'report> {
    line_number: u32,
    _command: &'report str,
    previous_error: &'report str,
}

struct NowFailingAssert<'report> {
    line_number: u32,
    _command: &'report str,
    new_error: &'report str,
}

struct NowDifferentErrorAssert<'report> {
    line_number: u32,
    _command: &'report str,
    old_error: &'report str,
    new_error: &'report str,
}

impl Display for FileDelta<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = self.ty.to_icon();
        write!(f, "| {icon} | ")?;

        write!(
            f,
            "[{filename}](https://github.com/WebAssembly/testsuite/blob/main/{filename}) | ",
            filename = sanitize_path(self.filepath)
        )?;

        match &self.ty {
            FileDeltaType::NewScriptError {
                old_num_passing_asserts,
                old_num_asserts,
                new_error,
                new_context,
                new_line_number,
            } => {
                let passing_percentage = percentage(*old_num_passing_asserts, *old_num_asserts);
                write_details_summary(
                    f,
                    |f| write!(f, "no longer compiles"),
                    |f| {
                        write!(f, "**Before**<br>")?;
                        write!(
                            f,
                            "Passing: {}/{} ({:.1}%)<br><br>",
                            old_num_passing_asserts, old_num_asserts, passing_percentage
                        )?;
                        write!(f, "**Now**<br>")?;
                        write!(
                            f,
                            "L. {}: {new_context}<br>",
                            to_str_or_placeholder(*new_line_number)
                        )?;
                        write!(f, "`{new_error}`<br>")
                    },
                )?;
                write!(f, " |")
            }
            FileDeltaType::ScriptErrorResolved {
                old_error,
                old_context,
                old_line_number,
                new_num_passing_asserts,
                new_num_asserts,
            } => {
                let passing_percentage = percentage(*new_num_passing_asserts, *new_num_asserts);

                write_details_summary(
                    f,
                    |f| write!(f, "compiles now"),
                    |f| {
                        write!(f, "**Before**<br>")?;
                        write!(
                            f,
                            "L. {}: {old_context}<br>",
                            to_str_or_placeholder(*old_line_number)
                        )?;
                        write!(f, "`{old_error}`<br>")
                    },
                )?;

                write!(
                    f,
                    " | {}/{} | {:.1}% |",
                    new_num_passing_asserts, new_num_asserts, passing_percentage
                )
            }
            FileDeltaType::ScriptErrorChanged {
                old_error,
                old_context,
                old_line_number,
                new_error,
                new_context,
                new_line_number,
            } => {
                write_details_summary(
                    f,
                    |f| write!(f, "still not compiling, error changed"),
                    |f| {
                        write!(f, "**Before**<br>")?;
                        write!(
                            f,
                            "L. {}: {old_context}<br>",
                            to_str_or_placeholder(*old_line_number)
                        )?;
                        write!(f, "`{old_error}`<br>")?;

                        write!(f, "**Now**<br>")?;
                        write!(
                            f,
                            "L. {}:{new_context}<br>",
                            to_str_or_placeholder(*new_line_number)
                        )?;
                        write!(f, "`{new_error}`<br>")
                    },
                )?;
                write!(f, " | |")
            }
            FileDeltaType::AssertsPassing {
                num_asserts,
                old_num_passing_asserts,
                now_passing,
            } => {
                write_details_summary(
                    f,
                    |f| write!(f, "{} passing", now_passing.len()),
                    |f| {
                        now_passing.iter().try_for_each(|assert| {
                            write!(
                                f,
                                "- L. {}: Previous error: `{}`<br>",
                                assert.line_number, assert.previous_error
                            )
                        })
                    },
                )?;

                let new_num_passing_asserts = *old_num_passing_asserts + now_passing.len();
                let old_passing_percentage = percentage(*old_num_passing_asserts, *num_asserts);
                let new_passing_percentage = percentage(new_num_passing_asserts, *num_asserts);
                let passing_percentage_delta = new_passing_percentage - old_passing_percentage;

                write!(
                    f,
                    " | {}/{} (+{}) | {:.1}% (+{:.1}%) |",
                    new_num_passing_asserts,
                    num_asserts,
                    now_passing.len(),
                    new_passing_percentage,
                    passing_percentage_delta
                )
            }
            FileDeltaType::AssertsFailing {
                num_asserts,
                old_num_passing_asserts,
                now_failing,
            } => {
                write_details_summary(
                    f,
                    |f| write!(f, "{} failing", now_failing.len()),
                    |f| {
                        now_failing.iter().try_for_each(|assert| {
                            write!(f, "- L. {}: `{}`<br>", assert.line_number, assert.new_error)
                        })
                    },
                )?;

                let new_num_passing_asserts = *old_num_passing_asserts - now_failing.len();
                let old_passing_percentage = percentage(*old_num_passing_asserts, *num_asserts);
                let new_passing_percentage = percentage(new_num_passing_asserts, *num_asserts);
                let passing_percentage_delta = new_passing_percentage - old_passing_percentage;

                write!(
                    f,
                    " | {}/{} (-{}) | {:.1}% (-{:.1}%) |",
                    new_num_passing_asserts,
                    num_asserts,
                    now_failing.len(),
                    new_passing_percentage,
                    -passing_percentage_delta
                )
            }
            FileDeltaType::AssertsChanged {
                now_passing,
                now_failing,
                now_different_error,
            } => {
                write_details_summary(
                    f,
                    |f| {
                        let mut has_written_already = false;
                        let mut maybe_insert_comma = || {
                            if std::mem::replace(&mut has_written_already, true) {
                                ", "
                            } else {
                                ""
                            }
                        };

                        if !now_failing.is_empty() {
                            write!(f, "{}{} failing", maybe_insert_comma(), now_failing.len())?;
                        }

                        if !now_passing.is_empty() {
                            write!(f, "{}{} passing", maybe_insert_comma(), now_passing.len())?;
                        }

                        if !now_different_error.is_empty() {
                            write!(
                                f,
                                "{}{} errors changed",
                                maybe_insert_comma(),
                                now_different_error.len()
                            )?;
                        }
                        Ok(())
                    },
                    |f| {
                        let mut has_written_already = false;
                        let mut maybe_insert_br = || {
                            if std::mem::replace(&mut has_written_already, true) {
                                "<br>"
                            } else {
                                ""
                            }
                        };
                        if !now_failing.is_empty() {
                            write!(f, "{}**Failing**<br>", maybe_insert_br())?;
                            now_failing.iter().try_for_each(|assert| {
                                write!(f, "- L. {}: `{}`<br>", assert.line_number, assert.new_error)
                            })?;
                        }

                        if !now_different_error.is_empty() {
                            write!(f, "{}**Errors changed**<br>", maybe_insert_br())?;
                            now_different_error.iter().try_for_each(|different_error| {
                                write!(
                                    f,
                                    "- L. {}: From `{}` to `{}`",
                                    different_error.line_number,
                                    different_error.old_error,
                                    different_error.new_error
                                )
                            })?;
                        }

                        if !now_passing.is_empty() {
                            write!(f, "{}**Passing**<br>", maybe_insert_br())?;
                            now_passing.iter().try_for_each(|assert| {
                                write!(
                                    f,
                                    "- L. {}: Previous error: `{}`<br>",
                                    assert.line_number, assert.previous_error
                                )
                            })?;
                        }

                        Ok(())
                    },
                )?;

                write!(f, " | | |")
            }
        }
    }
}

fn to_str_or_placeholder<T: ToString>(t: Option<T>) -> Cow<'static, str> {
    t.as_ref()
        .map(ToString::to_string)
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed("-"))
}

fn percentage(x: usize, total: usize) -> f32 {
    100.0 * x as f32 / total as f32
}
