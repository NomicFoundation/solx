//!
//! The `solc --standard-json` output contract EVM extra metadata.
//!

pub mod defined_function;

use std::collections::BTreeMap;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::IgnoredAny;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::ser::SerializeStruct;

use self::defined_function::DefinedFunction;

///
/// The `solc --standard-json` output contract EVM extra metadata.
///
#[derive(Debug, Default, Clone)]
pub struct ExtraMetadata {
    /// The defined functions keyed by their code segment and block tag.
    pub defined_functions: HashMap<(solx_utils::CodeSegment, u64), DefinedFunction>,
}

impl ExtraMetadata {
    ///
    /// Returns the function reference for the specified tag.
    ///
    pub fn get(
        &self,
        code_segment: solx_utils::CodeSegment,
        tag: &u64,
    ) -> Option<&DefinedFunction> {
        self.defined_functions.get(&(code_segment, *tag))
    }
}

impl Serialize for ExtraMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let functions = self
            .defined_functions
            .values()
            .map(|function| ((function.creation_tag, function.runtime_tag), function))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect::<Vec<_>>();
        let mut state = serializer.serialize_struct("ExtraMetadata", 1)?;
        state.serialize_field("recursiveFunctions", &functions)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ExtraMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExtraMetadataVisitor;

        impl<'de> Visitor<'de> for ExtraMetadataVisitor {
            type Value = ExtraMetadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("the EVM extra metadata object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut extra_metadata = ExtraMetadata::default();
                while let Some(key) = map.next_key::<String>()? {
                    if key != "recursiveFunctions" {
                        map.next_value::<IgnoredAny>()?;
                        continue;
                    }
                    for function in map.next_value::<Vec<DefinedFunction>>()? {
                        let creation_key = function
                            .creation_tag
                            .map(|tag| (solx_utils::CodeSegment::Deploy, tag as u64));
                        let runtime_key = function
                            .runtime_tag
                            .map(|tag| (solx_utils::CodeSegment::Runtime, tag as u64));
                        match (creation_key, runtime_key) {
                            (Some(creation_key), Some(runtime_key)) => {
                                extra_metadata
                                    .defined_functions
                                    .insert(creation_key, function.clone());
                                extra_metadata
                                    .defined_functions
                                    .insert(runtime_key, function);
                            }
                            (Some(key), None) | (None, Some(key)) => {
                                extra_metadata.defined_functions.insert(key, function);
                            }
                            (None, None) => {}
                        }
                    }
                }
                Ok(extra_metadata)
            }
        }

        deserializer.deserialize_map(ExtraMetadataVisitor)
    }
}
