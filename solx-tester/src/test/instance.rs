//!
//! The test contract instance used for building.
//!

use revm::primitives::Address;

///
/// The test contract instance used for building.
///
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub struct Instance {
    /// The contract path.
    pub path: String,
    /// The instance address.
    pub address: Option<Address>,
    /// Whether the instance is main.
    pub is_main: bool,
    /// Whether the instance is a library.
    pub is_library: bool,
    /// The deploy bytecode.
    pub deploy_code: Vec<u8>,
    /// The runtime bytecode size.
    pub runtime_code_size: usize,
}

impl Instance {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        path: String,
        address: Option<Address>,
        is_main: bool,
        is_library: bool,
        deploy_code: Vec<u8>,
        runtime_code_size: usize,
    ) -> Self {
        Self {
            path,
            address,
            is_main,
            is_library,
            deploy_code,
            runtime_code_size,
        }
    }

    ///
    /// Sets the address of the instance.
    ///
    pub fn set_address(&mut self, address: Address) {
        self.address = Some(address);
    }

    ///
    /// Returns the instance path if applicable.
    ///
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    ///
    /// Whether the instance is main.
    ///
    pub fn is_main(&self) -> bool {
        self.is_main
    }

    ///
    /// Whether the instance is a library.
    ///
    pub fn is_library(&self) -> bool {
        self.is_library
    }

    ///
    /// Returns the instance address if applicable.
    ///
    pub fn address(&self) -> Option<&Address> {
        self.address.as_ref()
    }
}
