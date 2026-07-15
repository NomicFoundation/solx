//!
//! The EVM deploy address iterator.
//!

use std::collections::HashMap;

use revm::primitives::Address;

///
/// The EVM deploy address iterator.
///
#[derive(Debug, Default, Clone)]
pub struct AddressIterator {
    /// Account nonces.
    nonces: HashMap<Address, usize>,
}

impl AddressIterator {
    ///
    /// Returns the next address.
    ///
    pub fn next(&mut self, caller: &Address, increment_nonce: bool) -> Address {
        let mut stream = rlp::RlpStream::new_list(2);
        stream.append(&caller.as_slice());
        stream.append(&self.nonce(caller));

        let hash = solx_utils::Keccak256Hash::from_slice(&stream.out());
        let address = Address::from_slice(
            &hash.as_bytes()[solx_utils::BYTE_LENGTH_FIELD - solx_utils::BYTE_LENGTH_ETH_ADDRESS..],
        );

        if increment_nonce {
            self.increment_nonce(caller);
        }

        address
    }

    ///
    /// Increments the nonce for the caller.
    ///
    pub fn increment_nonce(&mut self, caller: &Address) {
        let nonce = self.nonces.entry(*caller).or_insert(1);
        *nonce += 1;
    }

    ///
    /// Returns the nonce for the caller.
    ///
    /// If the nonce for the `caller` does not exist, it will be created.
    ///
    pub fn nonce(&mut self, caller: &Address) -> usize {
        *self.nonces.entry(*caller).or_insert(1)
    }
}
