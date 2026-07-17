//!
//! A worksheet of the benchmark workbook.
//!

///
/// A worksheet of the benchmark workbook, and the only enumeration of them:
/// the variants are the workbook's sheet order, and creation, totals, diffs,
/// and finalization all derive from `ALL`. Adding a sheet is one variant and
/// one `spec` arm; the compiler finds every other site.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sheet {
    /// Build failures per project.
    BuildFailures,
    /// Test failures per project.
    TestFailures,
    /// Runtime fee measurements.
    RuntimeFee,
    /// Deployment fee measurements.
    DeployFee,
    /// Runtime bytecode size measurements.
    RuntimeSize,
    /// Deploy bytecode size measurements.
    DeploySize,
    /// Compilation time measurements.
    CompilationTime,
    /// Testing time measurements.
    TestingTime,
}

impl Sheet {
    /// Every sheet, in workbook order, which is declaration order, since the
    /// variant's discriminant indexes its worksheet.
    pub const ALL: [Self; 8] = [
        Self::BuildFailures,
        Self::TestFailures,
        Self::RuntimeFee,
        Self::DeployFee,
        Self::RuntimeSize,
        Self::DeploySize,
        Self::CompilationTime,
        Self::TestingTime,
    ];

    /// The sheet's tab name and the headers preceding its value columns.
    pub fn spec(self) -> (&'static str, Vec<(&'static str, usize)>) {
        let project = ("Project", 15);
        let contract = ("Contract", 60);
        let function = ("Function", 40);
        match self {
            Self::BuildFailures => ("Build Failures", vec![project]),
            Self::TestFailures => ("Test Failures", vec![project]),
            Self::RuntimeFee => ("Runtime Gas", vec![project, contract, function]),
            Self::DeployFee => ("Deploy Gas", vec![project, contract]),
            Self::RuntimeSize => ("Runtime Size", vec![project, contract]),
            Self::DeploySize => ("Deploy Size", vec![project, contract]),
            Self::CompilationTime => ("Compilation Time", vec![project]),
            Self::TestingTime => ("Testing Time", vec![project]),
        }
    }
}
