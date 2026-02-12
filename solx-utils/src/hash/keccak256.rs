//!
//! Keccak-256 hash utilities.
//!

use sha3::Digest;

///
/// Keccak-256 hash utilities.
///
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Keccak256 {
    /// Binary representation.
    bytes: [u8; crate::BYTE_LENGTH_FIELD],
    /// Hexadecimal string representation.
    string: String,
}

impl Keccak256 {
    ///
    /// Computes the `keccak256` hash for `preimage`.
    ///
    pub fn from_slice(preimage: &[u8]) -> Self {
        let bytes = sha3::Keccak256::digest(preimage).into();
        let string = format!("0x{}", hex::encode(bytes));
        Self { bytes, string }
    }

    ///
    /// Returns a reference to the hexadecimal string representation of the IPFS hash.
    ///
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }

    ///
    /// Extracts the binary representation.
    ///
    pub fn to_vec(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
}

impl std::fmt::Display for Keccak256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
