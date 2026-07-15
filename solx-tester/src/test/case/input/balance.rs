//!
//! The balance check input variant.
//!

use std::sync::Arc;
use std::sync::Mutex;

use revm::DatabaseRef;
use revm::primitives::Address;
use revm::primitives::U256;

use crate::revm::REVM;
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
    address: Address,
    /// The expected balance.
    balance: U256,
}

impl Balance {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(address: Address, balance: U256) -> Self {
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
            .basic_ref(self.address)
            .map(|account_info| account_info.map(|info| info.balance).unwrap_or_default())
            .expect("Always valid");
        if balance == self.balance {
            Summary::passed_special(summary, test);
        } else {
            Summary::failed(
                summary,
                test,
                self.balance.into(),
                balance.into(),
                self.address.as_slice().to_vec(),
            );
        }
    }
}
