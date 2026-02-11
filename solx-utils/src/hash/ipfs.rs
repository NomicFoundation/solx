//!
//! IPFS hash utilities.
//!

use base58::FromBase58;

///
/// IPFS hash utilities.
///
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IPFS {
    /// Binary representation.
    #[serde(with = "serde_arrays")]
    bytes: [u8; 2 + crate::BYTE_LENGTH_FIELD],
    /// Base58 string representation.
    string_base58: String,
    /// Hexadecimal string representation.
    string_hex: String,
}

impl IPFS {
    ///
    /// Computes the IPFS hash for `preimage`.
    ///
    pub fn from_slice(preimage: &[u8]) -> Self {
        let hasher = ipfs_hasher::IpfsHasher::default();
        let string_base58 = hasher.compute(preimage);
        let bytes = string_base58
            .from_base58()
            .expect("Base58 conversion is always valid")
            .try_into()
            .expect("The size is always correct");
        let string_hex = hex::encode(bytes);
        Self {
            bytes,
            string_base58,
            string_hex,
        }
    }

    ///
    /// Extracts the binary representation.
    ///
    pub fn to_vec(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
}

impl std::fmt::Display for IPFS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.string_hex)
    }
}
