//!
//! LLVM sanitizers.
//!

///
/// LLVM sanitizers.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sanitizer {
    /// The address sanitizer.
    Address,
    /// The memory sanitizer.
    Memory,
    /// The memory with origins sanitizer.
    MemoryWithOrigins,
    /// The undefined behavior sanitizer
    Undefined,
    /// The thread sanitizer.
    Thread,
    /// The data flow sanitizer.
    DataFlow,
    /// Combine address and undefined behavior sanitizer.
    AddressUndefined,
}

impl std::str::FromStr for Sanitizer {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "address" => Ok(Self::Address),
            "memory" => Ok(Self::Memory),
            "memorywithorigins" => Ok(Self::MemoryWithOrigins),
            "undefined" => Ok(Self::Undefined),
            "thread" => Ok(Self::Thread),
            "dataflow" => Ok(Self::DataFlow),
            "address;undefined" => Ok(Self::AddressUndefined),
            value => Err(format!("Unsupported sanitizer: `{value}`")),
        }
    }
}

impl std::fmt::Display for Sanitizer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Address => write!(f, "Address"),
            Self::Memory => write!(f, "Memory"),
            Self::MemoryWithOrigins => write!(f, "MemoryWithOrigins"),
            Self::Undefined => write!(f, "Undefined"),
            Self::Thread => write!(f, "Thread"),
            Self::DataFlow => write!(f, "DataFlow"),
            Self::AddressUndefined => write!(f, "Address;Undefined"),
        }
    }
}
