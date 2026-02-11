//!
//! CBOR utilities.
//!

///
/// CBOR payload.
///
/// Used for encoding IPFS contract metadata hash.
///
#[derive(Debug, Clone, PartialEq)]
pub struct CBOR<'a, S>
where
    S: ToString,
{
    /// Hash type name and hash itself in binary representation.
    pub hash: Option<(S, &'a [u8])>,
    /// Key of the version field.
    pub version_key: String,
    /// Version data to be encoded in the `version_key` field.
    pub version_data: Vec<(String, semver::Version)>,
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
        version_key: String,
        version_data: Vec<(String, semver::Version)>,
    ) -> Self {
        assert!(!version_data.is_empty(), "Version data cannot be empty");

        Self {
            hash,
            version_key,
            version_data,
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

        cbor.push(0x60_u8 + (self.version_key.len() as u8));
        cbor.extend(self.version_key.as_bytes());
        let version_data = self
            .version_data
            .iter()
            .map(|(name, version)| format!("{name}:{version}"))
            .collect::<Vec<String>>()
            .join(";");
        cbor.push(0x78_u8);
        cbor.push(version_data.len() as u8);
        cbor.extend(version_data.as_bytes());

        cbor.extend((cbor.len() as u16).to_be_bytes());
        cbor
    }
}
