//!
//! IPFS hash utilities.
//!

use ipfs_unixfs::file::adder::FileAdder;

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
        let mut adder = FileAdder::default();
        let mut root = None;
        let mut written = 0;
        while written < preimage.len() {
            let (blocks, pushed) = adder.push(&preimage[written..]);
            root = blocks.last().map(|(cid, _)| cid);
            written += pushed;
        }
        // An input of exactly one chunk emits its root from `push`, leaving `finish` empty.
        let root = adder
            .finish()
            .last()
            .map(|(cid, _)| cid)
            .or(root)
            .expect("the file adder emits a root block for every input");
        let string_base58 = root.to_string();
        let bytes = root
            .to_bytes()
            .try_into()
            .expect("a CIDv0 is a 34-byte SHA-256 multihash");
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

#[cfg(test)]
mod tests {
    use super::IPFS;

    /// A Solidity source of exactly `size` bytes, so the expected hashes can be taken from the
    /// `dweb:/ipfs/` URL solc puts into the metadata for it.
    fn source(size: usize) -> Vec<u8> {
        let head = "contract C{}//";
        format!("{head}{}\n", "x".repeat(size - head.len() - 1)).into_bytes()
    }

    #[test]
    fn hashes_one_partial_chunk() {
        assert_eq!(
            IPFS::from_slice(&source(100)).string_base58,
            "QmXrKgSxFj667mufDfVHrNHXhNFEx2yueBh7Qd7myZEhVi"
        );
    }

    #[test]
    fn hashes_exactly_one_chunk() {
        assert_eq!(
            IPFS::from_slice(&source(262144)).string_base58,
            "QmSdGxUBTtR1iuis2kB2czNrWhxJjLcHx6zCboLa3duNHB"
        );
    }

    #[test]
    fn hashes_one_byte_past_a_chunk() {
        assert_eq!(
            IPFS::from_slice(&source(262145)).string_base58,
            "QmTdEtYCeHcYoPA8L2w9i5KjLgtou3KqZRvci4EfW4DWRY"
        );
    }

    #[test]
    fn hashes_exactly_two_chunks() {
        assert_eq!(
            IPFS::from_slice(&source(524288)).string_base58,
            "QmRyuSFwU9RqvZC4LmcpVRnX5Zfv3chSuSgdShY5Szeq4d"
        );
    }
}
