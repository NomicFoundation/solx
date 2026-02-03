//!
//! The test case.
//!

pub mod input;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::compilers::mode::Mode;
use crate::directories::matter_labs::test::metadata::case::Case as MatterLabsTestCase;
use crate::revm::REVM;
use crate::summary::Summary;
use crate::test::context::case::CaseContext;
use crate::test::context::input::InputContext;
use crate::test::instance::Instance;

use self::input::Input;

///
/// The test case.
///
#[derive(Debug, Clone)]
pub struct Case {
    /// The case name.
    name: Option<String>,
    /// The case inputs.
    inputs: Vec<Input>,
}

impl Case {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(name: Option<String>, inputs: Vec<Input>) -> Self {
        Self { name, inputs }
    }

    ///
    /// Try convert from Matter Labs compiler test metadata case.
    ///
    pub fn try_from_matter_labs(
        case: MatterLabsTestCase,
        mode: &Mode,
        instances: &BTreeMap<String, Instance>,
        method_identifiers: &Option<BTreeMap<String, BTreeMap<String, u32>>>,
    ) -> anyhow::Result<Self> {
        let mut inputs = Vec::with_capacity(case.inputs.len());

        for (index, input) in case.inputs.into_iter().enumerate() {
            let input = Input::try_from_matter_labs(input, mode, instances, method_identifiers)
                .map_err(|error| anyhow::anyhow!("Input #{index} is invalid: {error}"))?;
            inputs.push(input);
        }

        Ok(Self::new(Some(case.name), inputs))
    }

    ///
    /// Try convert from Ethereum compiler test metadata case.
    ///
    pub fn try_from_ethereum(
        case: &[solx_solc_test_adapter::FunctionCall],
        instances: BTreeMap<String, Instance>,
        last_source: &str,
    ) -> anyhow::Result<Self> {
        let mut inputs = Vec::with_capacity(case.len());
        let mut caller =
            solx_solc_test_adapter::account_address(solx_solc_test_adapter::DEFAULT_ACCOUNT_INDEX);

        for (index, input) in case.iter().enumerate() {
            match input {
                solx_solc_test_adapter::FunctionCall::Account { input, .. } => {
                    caller = solx_solc_test_adapter::account_address(*input);
                }
                input => {
                    if let Some(input) =
                        Input::try_from_ethereum(input, &instances, last_source, &caller).map_err(
                            |error| anyhow::anyhow!("Failed to proccess input #{index}: {error}"),
                        )?
                    {
                        inputs.push(input);
                    }
                }
            }
        }

        Ok(Self::new(None, inputs))
    }

    ///
    /// Runs the case on REVM.
    ///
    pub fn run_revm(self, summary: Arc<Mutex<Summary>>, context: &CaseContext, mut vm: REVM) {
        for (index, input) in self.inputs.into_iter().enumerate() {
            let context = InputContext {
                case_context: context,
                case_name: &self.name,
                selector: index,
            };
            input.run_revm(summary.clone(), &mut vm, context)
        }
    }
}
