//!
//! CBOR utilities.
//!

///
/// CBOR payload.
///
/// Used for encoding IPFS contract metadata hash.
///
/// The compiler entry is keyed by the compiler's own name, since verifiers read a `solc` entry as
/// the solc version, and is a byte string, since verifiers such as Sourcify hex-encode the values
/// of keys they do not know and fail on a text value.
///
#[derive(Debug, Clone, PartialEq)]
pub struct CBOR<'a, S>
where
    S: ToString,
{
    /// Hash type name and hash itself in binary representation.
    pub hash: Option<(S, &'a [u8])>,
    /// The compiler name, the key of its version entry.
    pub compiler_name: &'a str,
    /// The compiler version, encoded as its major, minor and patch bytes.
    pub compiler_version: semver::Version,
}

impl<'a, S> CBOR<'a, S>
where
    S: ToString,
{
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        hash: Option<(S, &'a [u8])>,
        compiler_name: &'a str,
        compiler_version: semver::Version,
    ) -> Self {
        Self {
            hash,
            compiler_name,
            compiler_version,
        }
    }

    ///
    /// Returns a CBOR-encoded vector.
    ///
    pub fn to_vec(&self) -> Vec<u8> {
        let field_count = (self.hash.is_some() as usize) + 1;
        let mut cbor = Vec::with_capacity(64);
        cbor.push(0xA0_u8 + (field_count as u8));

        if let Some((r#type, hash)) = self.hash.as_ref() {
            cbor.push(0x64_u8);
            cbor.extend(r#type.to_string().as_bytes());
            cbor.push(0x58_u8);
            cbor.push(hash.len() as u8);
            cbor.extend_from_slice(hash);
        }

        cbor.push(0x60_u8 + (self.compiler_name.len() as u8));
        cbor.extend(self.compiler_name.as_bytes());
        cbor.push(0x43_u8);
        cbor.extend(
            [
                self.compiler_version.major,
                self.compiler_version.minor,
                self.compiler_version.patch,
            ]
            .map(|component| {
                u8::try_from(component).expect("a CBOR version component fits in one byte")
            }),
        );

        cbor.extend((cbor.len() as u16).to_be_bytes());
        cbor
    }
}

#[cfg(test)]
mod tests {
    use super::CBOR;
    use crate::MetadataHashType;

    #[test]
    fn encodes_version_without_hash() {
        let cbor = CBOR::<MetadataHashType>::new(None, "slang", semver::Version::new(1, 2, 3));
        assert_eq!(hex::encode(cbor.to_vec()), "a165736c616e6743010203000b");
    }

    #[test]
    fn encodes_version_after_hash() {
        let hash = [0xab_u8; 34];
        let cbor = CBOR::new(
            Some((MetadataHashType::IPFS, hash.as_slice())),
            "slang",
            semver::Version::new(1, 2, 3),
        );
        assert_eq!(
            hex::encode(cbor.to_vec()),
            format!(
                "a264697066735822{}65736c616e67430102030034",
                "ab".repeat(34)
            )
        );
    }
}
