use anyhow::Context as _;
use ci_reports::CIFullReport;
use clap::Parser;
use deltas::FileDeltas;
use std::{fmt::Display, fs::File, io::BufReader, path::PathBuf};
use summary::ReportSummary;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Generate a Markdown report for the differences of testsuite results
struct Args {
    /// Path to the testsuite output JSON for the old version
    old_results_path: PathBuf,

    /// Path to the testsuite output JSON for the new version
    new_results_path: PathBuf,
}

mod ci_reports;
mod deltas;
mod summary;

fn main() -> anyhow::Result<()> {
    // Read arguments
    let args = Args::parse();
    let old_file = File::open(args.old_results_path).context("failed to open old results file")?;
    let new_file = File::open(args.new_results_path).context("failed to open new results file")?;

    // Read & parse the data
    let old_report: CIFullReport = serde_json::from_reader(BufReader::new(old_file))
        .context("failed to parse old file contents")?;
    let new_report: CIFullReport = serde_json::from_reader(BufReader::new(new_file))
        .context("failed to parse new file contents")?;

    // Analyze the data
    let deltas =
        deltas::generate(&old_report, &new_report).context("failed to generate file deltas")?;
    let summary = summary::generate(&new_report);
    let report = TestsuiteReport { deltas, summary };

    // Generate the final report
    println!("{report}");

    Ok(())
}

struct TestsuiteReport<'report> {
    deltas: FileDeltas<'report>,
    summary: ReportSummary<'report>,
}

impl Display for TestsuiteReport<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "# 🗒️ [WebAssembly Testsuite](https://github.com/WebAssembly/testsuite) Report"
        )?;
        writeln!(f)?;

        writeln!(f, "{}", self.deltas)?;
        writeln!(f, "{}", self.summary)
    }
}

pub fn sanitize_path(path: &str) -> &str {
    path.trim_start_matches("./tests/specification/testsuite/")
}

pub fn write_details_summary<W: std::fmt::Write>(
    writer: &mut W,
    summary: impl FnOnce(&mut W) -> std::fmt::Result,
    contents: impl FnOnce(&mut W) -> std::fmt::Result,
) -> std::fmt::Result {
    write!(writer, "<details><summary>")?;
    summary(writer)?;
    write!(writer, "</summary>")?;
    contents(writer)?;
    write!(writer, "</details>")
}
