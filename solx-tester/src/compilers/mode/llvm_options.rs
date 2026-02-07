//!
//! `solx` tester LLVM options.
//!

use std::sync::Mutex;

use solx_utils::SyncLock;

///
/// `solx` tester LLVM options.
///
#[derive(Debug, Default, Clone)]
pub struct LLVMOptions {
    /// Whether the LLVM `verify each` option is enabled.
    is_verify_each_enabled: bool,
    /// Whether the LLVM `debug logging` option is enabled.
    is_debug_logging_enabled: bool,
}

/// The one-time initialization cell for the global variable.
static LLVM_OPTIONS: once_cell::sync::OnceCell<LLVMOptions> = once_cell::sync::OnceCell::new();

/// The mutex to allow simultaneous access to only one target machine.
static LLVM_OPTIONS_LOCK: Mutex<()> = Mutex::new(());

impl LLVMOptions {
    ///
    /// A shortcut constructor with lazy initialization.
    ///
    pub fn initialize(
        is_verify_each_enabled: bool,
        is_debug_logging_enabled: bool,
    ) -> anyhow::Result<()> {
        let _ = LLVM_OPTIONS.get_or_try_init(|| -> anyhow::Result<LLVMOptions> {
            Ok(Self {
                is_verify_each_enabled,
                is_debug_logging_enabled,
            })
        })?;
        Ok(())
    }

    ///
    /// A shortcut constructor with lazy initialization.
    ///
    pub fn get() -> LLVMOptions {
        LLVM_OPTIONS
            .get_or_try_init(|| -> anyhow::Result<LLVMOptions> { Ok(Self::default()) })
            .cloned()
            .expect("Always exists")
    }

    ///
    /// Whether the LLVM `verify each` option is enabled.
    ///
    pub fn is_verify_each_enabled(&self) -> bool {
        let _guard = LLVM_OPTIONS_LOCK.lock_sync();

        self.is_verify_each_enabled
    }

    ///
    /// Whether the LLVM `debug logging` option is enabled.
    ///
    pub fn is_debug_logging_enabled(&self) -> bool {
        let _guard = LLVM_OPTIONS_LOCK.lock_sync();

        self.is_debug_logging_enabled
    }
}
