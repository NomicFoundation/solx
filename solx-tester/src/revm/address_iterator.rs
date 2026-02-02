//!
//! The EVM deploy address iterator.
//!

use std::collections::HashMap;
use std::str::FromStr;

///
/// The EVM deploy address iterator.
///
#[derive(Debug, Default, Clone)]
pub struct AddressIterator {
    /// Account nonces.
    nonces: HashMap<web3::types::Address, usize>,
}

impl AddressIterator {
    ///
    /// Returns the next address.
    ///
    pub fn next(
        &mut self,
        caller: &web3::types::Address,
        increment_nonce: bool,
    ) -> web3::types::Address {
        let mut stream = rlp::RlpStream::new_list(2);
        stream.append(&caller.as_bytes());
        stream.append(&self.nonce(caller));

        let hash = solx_utils::Keccak256Hash::from_slice(&stream.out());
        let address = web3::types::Address::from_str(
            &hash.to_string()
                [2 + 2 * (solx_utils::BYTE_LENGTH_FIELD - solx_utils::BYTE_LENGTH_ETH_ADDRESS)..],
        )
        .expect("Always valid");

        if increment_nonce {
            self.increment_nonce(caller);
        }

        address
    }

    ///
    /// Increments the nonce for the caller.
    ///
    pub fn increment_nonce(&mut self, caller: &web3::types::Address) {
        let nonce = self.nonces.entry(*caller).or_insert(1);
        *nonce += 1;
    }

    ///
    /// Returns the nonce for the caller.
    ///
    /// If the nonce for the `caller` does not exist, it will be created.
    ///
    pub fn nonce(&mut self, caller: &web3::types::Address) -> usize {
        *self.nonces.entry(*caller).or_insert(1)
    }
}
