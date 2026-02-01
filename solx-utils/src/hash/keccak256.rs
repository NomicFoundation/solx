//!
//! Keccak-256 hash utilities.
//!

use sha3::Digest;
use sha3::digest::FixedOutput;

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
            "0x8b904e9a94975df70f5804cc15ba7d249cd814144885fecb68de56c7f5d1e627"
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
            "0xc838553f6bb8ac30851f970fa540b2248a9ec0239a02eeebbb1c6a9c2f649be4"
        );
    }

    #[test]
    fn display() {
        assert_eq!(
            super::Keccak256::from_slice("solx".as_bytes()).to_string(),
            "0x8b904e9a94975df70f5804cc15ba7d249cd814144885fecb68de56c7f5d1e627"
        );
    }
}
