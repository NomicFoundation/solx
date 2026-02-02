//!
//! The storage emptiness check input variant.
//!

use std::sync::Arc;
use std::sync::Mutex;

use crate::revm::REVM;
use crate::summary::Summary;
use crate::test::InputContext;
use crate::test::case::input::identifier::InputIdentifier;
use crate::test::description::TestDescription;

///
/// The storage emptiness check input variant.
///
#[derive(Debug, Clone)]
pub struct StorageEmpty {
    /// Whether storage is empty.
    is_empty: bool,
}

impl StorageEmpty {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(is_empty: bool) -> Self {
        Self { is_empty }
    }
}

impl StorageEmpty {
    ///
    /// Runs the storage empty check on REVM.
    ///
    pub fn run_revm(self, summary: Arc<Mutex<Summary>>, vm: &mut REVM, context: InputContext<'_>) {
        let input_index = context.selector;
        let test =
            TestDescription::from_context(context, InputIdentifier::StorageEmpty { input_index });
        let mut is_empty = true;
        for cache_account in vm.ctx_db().cache.accounts.values() {
            let plain_account = cache_account.clone().account;
            if let Some(plain_account) = plain_account {
                for (_, value) in plain_account.storage.iter() {
                    if !value.is_zero() {
                        is_empty = false;
                    }
                }
            }
        }

        if is_empty == self.is_empty {
            Summary::passed_special(summary, test);
        } else {
            Summary::failed(summary, test, self.is_empty.into(), is_empty.into(), vec![]);
        }
    }
}
