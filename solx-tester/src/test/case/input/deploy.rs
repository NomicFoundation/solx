//!
//! The EVM deploy contract call input variant.
//!

use std::sync::Arc;
use std::sync::Mutex;

use revm::context::result::ExecutionResult;

use crate::summary::Summary;
use crate::test::InputContext;
use crate::test::case::input::calldata::Calldata;
use crate::test::case::input::identifier::InputIdentifier;
use crate::test::case::input::output::Output;
use crate::test::description::TestDescription;

use crate::revm::REVM;
use crate::revm::revm_type_conversions::revm_bytes_to_vec_value;

///
/// The EVM deploy contract call input variant.
///
#[derive(Debug, Clone)]
pub struct Deploy {
    /// The contract identifier.
    identifier: String,
    /// The contract deploy code.
    deploy_code: Vec<u8>,
    /// The contract runtime code size.
    runtime_code_size: usize,
    /// The calldata.
    calldata: Calldata,
    /// The caller.
    caller: web3::types::Address,
    /// The value in wei.
    value: Option<u128>,
    /// The expected output.
    expected: Output,
}

impl Deploy {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        identifier: String,
        deploy_code: Vec<u8>,
        runtime_code_size: usize,
        calldata: Calldata,
        caller: web3::types::Address,
        value: Option<u128>,
        expected: Output,
    ) -> Self {
        Self {
            identifier,
            deploy_code,
            runtime_code_size,
            calldata,
            caller,
            value,
            expected,
        }
    }
}

impl Deploy {
    ///
    /// Runs the deploy transaction on native REVM.
    ///
    pub fn run_revm(self, summary: Arc<Mutex<Summary>>, vm: &mut REVM, context: InputContext<'_>) {
        let input_index = context.selector;
        let test = TestDescription::from_context(
            context,
            InputIdentifier::Deployer {
                contract_identifier: self.identifier.clone(),
            },
        );

        let deploy_code_size = self.deploy_code.len();
        let mut calldata = self.deploy_code;
        calldata.extend(self.calldata.inner);
        let calldata_cost = REVM::calldata_gas_cost(calldata.as_slice());

        let tx = REVM::new_deploy_transaction(self.caller, self.value, calldata.clone());

        let initial_balance = (web3::types::U256::from(1) << 100)
            + web3::types::U256::from(self.value.unwrap_or_default());
        vm.set_account(&self.caller, initial_balance);
        vm.set_block_data(
            revm::primitives::U256::from(input_index + 1),
            revm::primitives::U256::from(((input_index + 1) as u128) * REVM::BLOCK_TIMESTAMP_STEP),
        );

        let result = match vm.execute_transaction(tx) {
            Ok(result) => result,
            Err(error) => {
                Summary::invalid(summary.clone(), test, error);
                return;
            }
        };

        let (output, total_gas_used, halt_reason) = match result {
            ExecutionResult::Success {
                reason: _,
                gas_used,
                gas_refunded: _,
                logs,
                output,
            } => ((output, logs).into(), gas_used, None),
            ExecutionResult::Revert { gas_used, output } => {
                let return_data_value = revm_bytes_to_vec_value(output);
                (Output::new(return_data_value, true, vec![]), gas_used, None)
            }
            ExecutionResult::Halt { reason, gas_used } => {
                (Output::new(vec![], true, vec![]), gas_used, Some(reason))
            }
        };

        let gas = REVM::deploy_bytecode_execution_gas(
            total_gas_used,
            calldata_cost,
            deploy_code_size,
            self.runtime_code_size,
        );

        if output == self.expected {
            Summary::passed_deploy(
                summary,
                test,
                deploy_code_size as u64,
                self.runtime_code_size as u64,
                gas,
            );
        } else if let Some(error) = halt_reason {
            Summary::invalid(summary, test, format!("{error:?}"));
        } else {
            Summary::failed(summary, test, self.expected, output, calldata);
        }
    }
}
