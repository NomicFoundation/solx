//!
//! Per-context capture of LLVM diagnostics emitted by the EVM backend.
//!

use std::cell::Cell;
use std::cell::RefCell;
use std::ffi::c_void;

use inkwell::context::AsContextRef;
use inkwell::llvm_sys::LLVMDiagnosticSeverity;
use inkwell::llvm_sys::core::LLVMContextSetDiagnosticHandler;
use inkwell::llvm_sys::core::LLVMDisposeMessage;
use inkwell::llvm_sys::core::LLVMGetDiagInfoDescription;
use inkwell::llvm_sys::core::LLVMGetDiagInfoSeverity;
use inkwell::llvm_sys::prelude::LLVMBool;
use inkwell::llvm_sys::prelude::LLVMDiagnosticInfoRef;

unsafe extern "C" {
    ///
    /// The EVM-local C API accessor for the stack-region-overflow diagnostic payload.
    ///
    fn LLVMGetDiagInfoEVMStackRegionOverflow(
        info: LLVMDiagnosticInfoRef,
        total_stack_size: *mut u64,
        stack_region_size: *mut u64,
    ) -> LLVMBool;
}

///
/// The stack-region-overflow report captured from an EVM backend diagnostic.
///
/// Surfaced as a typed error so the driver can retry codegen with
/// `total_stack_size` as the new spill area size.
///
#[derive(Debug, Clone, Copy)]
pub struct StackRegionOverflow {
    /// The total stack size the module requires.
    pub total_stack_size: u64,
    /// The stack region size the module was compiled with.
    pub stack_region_size: u64,
    /// Whether the overflowing pass was the size fallback.
    pub is_size_fallback: bool,
}

impl std::fmt::Display for StackRegionOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "total stack size ({}) exceeds the allocated stack region size ({})",
            self.total_stack_size, self.stack_region_size
        )
    }
}

impl std::error::Error for StackRegionOverflow {}

///
/// The diagnostics recorded by the installed handler.
///
#[derive(Debug, Default)]
struct Captured {
    /// The last stack-region-overflow report.
    overflow: Cell<Option<StackRegionOverflow>>,
    /// The first non-overflow error diagnostic.
    error: RefCell<Option<String>>,
}

///
/// Captures EVM backend diagnostics on an LLVM context.
///
/// Installing replaces LLVM's default handler, which exits the process on
/// error diagnostics, so `check` must be called after every emission.
/// Dropping uninstalls the handler.
///
pub(crate) struct Capture<'ctx> {
    /// The context the handler is installed on.
    llvm: &'ctx inkwell::context::Context,
    /// The recorded diagnostics the installed handler writes to.
    captured: Box<Captured>,
}

impl<'ctx> Capture<'ctx> {
    ///
    /// Installs the capturing handler on `llvm`.
    ///
    pub fn install(llvm: &'ctx inkwell::context::Context) -> Self {
        let captured = Box::<Captured>::default();
        unsafe {
            LLVMContextSetDiagnosticHandler(
                llvm.as_ctx_ref(),
                Some(handle),
                std::ptr::from_ref::<Captured>(captured.as_ref())
                    .cast_mut()
                    .cast::<c_void>(),
            );
        }
        Self { llvm, captured }
    }

    ///
    /// Returns the error diagnostic recorded since the last check, if any.
    ///
    /// A stack-region-overflow report becomes the typed `StackRegionOverflow`
    /// error the driver retries on, tagged with `is_size_fallback` so the
    /// driver knows which pass overflowed.
    ///
    pub fn check(&self, is_size_fallback: bool) -> anyhow::Result<()> {
        if let Some(mut overflow) = self.captured.overflow.take() {
            overflow.is_size_fallback = is_size_fallback;
            return Err(anyhow::Error::new(overflow));
        }
        if let Some(error) = self.captured.error.borrow_mut().take() {
            anyhow::bail!("LLVM diagnostic: {error}");
        }
        Ok(())
    }
}

impl Drop for Capture<'_> {
    fn drop(&mut self) {
        unsafe {
            LLVMContextSetDiagnosticHandler(self.llvm.as_ctx_ref(), None, std::ptr::null_mut());
        }
    }
}

///
/// Records stack-region-overflow reports and other error diagnostics, and
/// forwards the rest to `stderr` in place of LLVM's default handler.
///
extern "C" fn handle(info: LLVMDiagnosticInfoRef, captured: *mut c_void) {
    let captured = unsafe { &*captured.cast_const().cast::<Captured>() };

    let mut total_stack_size = 0u64;
    let mut stack_region_size = 0u64;
    let is_overflow = unsafe {
        LLVMGetDiagInfoEVMStackRegionOverflow(info, &mut total_stack_size, &mut stack_region_size)
    } != 0;
    if is_overflow {
        captured.overflow.set(Some(StackRegionOverflow {
            total_stack_size,
            stack_region_size,
            is_size_fallback: false,
        }));
        return;
    }

    let severity = unsafe { LLVMGetDiagInfoSeverity(info) };
    if !matches!(
        severity,
        LLVMDiagnosticSeverity::LLVMDSError | LLVMDiagnosticSeverity::LLVMDSWarning
    ) {
        return;
    }

    let description = unsafe { LLVMGetDiagInfoDescription(info) };
    let message = unsafe { std::ffi::CStr::from_ptr(description) }
        .to_string_lossy()
        .into_owned();
    unsafe { LLVMDisposeMessage(description) };

    if let LLVMDiagnosticSeverity::LLVMDSError = severity {
        let mut error = captured.error.borrow_mut();
        if error.is_none() {
            *error = Some(message);
        }
        return;
    }
    eprintln!("LLVM warning: {message}");
}
