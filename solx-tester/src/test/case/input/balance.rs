//!
//! The balance check input variant.
//!

use std::sync::Arc;
use std::sync::Mutex;

use revm::DatabaseRef;

use crate::revm::REVM;
use crate::revm::revm_type_conversions::web3_address_to_revm_address;
use crate::summary::Summary;
use crate::test::case::input::identifier::InputIdentifier;
use crate::test::context::input::InputContext;
use crate::test::description::TestDescription;

///
/// The balance check input variant.
///
#[derive(Debug, Clone)]
pub struct Balance {
    /// The account address.
    address: web3::types::Address,
    /// The expected balance.
    balance: web3::types::U256,
}

impl Balance {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(address: web3::types::Address, balance: web3::types::U256) -> Self {
        Self { address, balance }
    }
}

impl Balance {
    ///
    /// Runs the balance check on REVM.
    ///
    pub fn run_revm(self, summary: Arc<Mutex<Summary>>, vm: &mut REVM, context: InputContext<'_>) {
        let input_index = context.selector;
        let test = TestDescription::from_context(context, InputIdentifier::Balance { input_index });
        let balance = vm
            .db()
            .basic_ref(web3_address_to_revm_address(&self.address))
            .map(|account_info| account_info.map(|info| info.balance).unwrap_or_default())
            .expect("Always valid");
        let balance = web3::types::U256::from(balance.to_be_bytes());
        if balance == self.balance {
            Summary::passed_special(summary, test);
        } else {
            Summary::failed(
                summary,
                test,
                self.balance.into(),
                balance.into(),
                self.address.to_fixed_bytes().to_vec(),
            );
        }
    }
}
