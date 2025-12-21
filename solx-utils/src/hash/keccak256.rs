//!
//! Keccak-256 hash utilities.
//!

use sha3::digest::FixedOutput;
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
    /// Computes the `keccak256` hash for an array of `preimages`.
    ///
    pub fn from_slices<R: AsRef<[u8]>>(preimages: &[R]) -> Self {
        let mut hasher = sha3::Keccak256::new();
        for preimage in preimages.iter() {
            hasher.update(preimage);
        }
        let bytes: [u8; crate::BYTE_LENGTH_FIELD] = hasher.finalize_fixed().into();
        let string = format!("0x{}", hex::encode(bytes));
        Self { bytes, string }
    }

    ///
    /// Returns a reference to the 32-byte SHA-3 hash.
    ///
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
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

#[cfg(test)]
mod tests {
    #[test]
    fn single() {
        assert_eq!(
            super::Keccak256::from_slice("solx".as_bytes()).as_str(),
            "0x0238fb1ab06c28c32885f9a4842207ac480c2467df26b6c58e201679628c5a5b"
        );
    }

    #[test]
    fn multiple() {
        assert_eq!(
            super::Keccak256::from_slices(&[
                "solx".as_bytes(),
                "the".as_bytes(),
                "best".as_bytes()
            ])
            .as_str(),
            "0x30277e6e189b3fa474437d451ccbb2409c3b67fda53c6ad5df3f8f0f3873ff6b"
        );
    }

    #[test]
    fn display() {
        assert_eq!(
            super::Keccak256::from_slice("solx".as_bytes()).to_string(),
            "0x0238fb1ab06c28c32885f9a4842207ac480c2467df26b6c58e201679628c5a5b"
        );
    }
}
