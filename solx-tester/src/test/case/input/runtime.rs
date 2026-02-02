//!
//! The contract call input variant.
//!

use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use revm::context::result::ExecutionResult;

use crate::revm::REVM;
use crate::revm::revm_type_conversions::revm_bytes_to_vec_value;
use crate::summary::Summary;
use crate::test::case::input::calldata::Calldata;
use crate::test::case::input::identifier::InputIdentifier;
use crate::test::case::input::output::Output;
use crate::test::case::input::storage::Storage;
use crate::test::context::input::InputContext;
use crate::test::description::TestDescription;

///
/// The contract call input variant.
#[derive(Debug, Clone)]
pub struct Runtime {
    /// The input name.
    name: String,
    /// The address.
    address: web3::types::Address,
    /// The calldata.
    calldata: Calldata,
    /// The caller.
    caller: web3::types::Address,
    /// The value in wei.
    value: Option<u128>,
    /// The contracts storage to set before running.
    storage: Storage,
    /// The expected output.
    expected: Output,
}

impl Runtime {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        name: String,
        address: web3::types::Address,
        calldata: Calldata,
        caller: web3::types::Address,
        value: Option<u128>,
        storage: Storage,
        expected: Output,
    ) -> Self {
        Self {
            name,
            address,
            calldata,
            caller,
            value,
            storage,
            expected,
        }
    }
}

impl Runtime {
    ///
    /// Runs the call on REVM.
    ///
    pub fn run_revm(
        mut self,
        summary: Arc<Mutex<Summary>>,
        vm: &mut REVM,
        context: InputContext<'_>,
    ) {
        let input_index = context.selector;
        let test = TestDescription::from_context(
            context,
            InputIdentifier::Runtime {
                input_index,
                name: self.name,
            },
        );

        if test.selector.path == "solx-solidity/test/libsolidity/semanticTests/state/tx_origin.sol"
        {
            self.caller = web3::types::Address::from_str(REVM::TX_ORIGIN).unwrap();
        }
        let tx = REVM::new_runtime_transaction(
            self.address,
            self.caller,
            self.calldata.clone(),
            self.value,
        );

        let mut initial_balance = web3::types::U256::from(self.value.unwrap_or_default());
        if REVM::get_rich_addresses().contains(&self.caller) {
            initial_balance += web3::types::U256::from(1) << 100;
        }
        vm.set_account(&self.caller, initial_balance);

        let storage = self
            .storage
            .inner
            .get(&self.address)
            .cloned()
            .unwrap_or_default();
        vm.extend_account_storage(&self.address, storage);
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

        let (output, total_gas_used, error) = match result {
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

        let calldata_cost = REVM::calldata_gas_cost(self.calldata.inner.as_slice());
        let gas = REVM::runtime_bytecode_execution_gas(total_gas_used, calldata_cost);

        if output == self.expected {
            Summary::passed_runtime(summary, test, gas);
        } else if let Some(error) = error {
            Summary::invalid(summary, test, format!("{error:?}"));
        } else {
            Summary::failed(summary, test, self.expected, output, self.calldata.inner);
        }
    }
}
