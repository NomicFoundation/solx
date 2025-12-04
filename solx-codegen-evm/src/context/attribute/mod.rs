//!
//! The LLVM attribute.
//!

pub mod memory;

///
/// The LLVM attribute.
///
/// In order to check the real order in a new major version of LLVM, find the `Attributes.inc` file
/// inside of the LLVM build directory. This order is actually generated during the building.
///
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Attribute {
    /// Unused.
    Unused = 0,
    /// The eponymous LLVM attribute.
    AllocAlign,
    /// The eponymous LLVM attribute.
    AllocatedPointer,
    /// The eponymous LLVM attribute.
    AlwaysInline,
    /// The eponymous LLVM attribute.
    Builtin,
    /// The eponymous LLVM attribute.
    Cold = 5,
    /// The eponymous LLVM attribute.
    Convergent,
    /// The eponymous LLVM attribute.
    CoroDestroyOnlyWhenComplete,
    /// The eponymous LLVM attribute.
    CoroElideSafe,
    /// The eponymous LLVM attribute.
    DeadOnReturn,
    /// The eponymous LLVM attribute.
    DeadOnUnwind = 10,
    /// The eponymous LLVM attribute.
    DisableSanitizerInstrumentation,
    /// The eponymous LLVM attribute.
    FnRetThunkExtern,
    /// The eponymous LLVM attribute.
    Hot,
    /// The eponymous LLVM attribute.
    HybridPatchable,
    /// The eponymous LLVM attribute.
    ImmArg = 15,
    /// The eponymous LLVM attribute.
    InReg,
    /// The eponymous LLVM attribute.
    InlineHint,
    /// The eponymous LLVM attribute.
    JumpTable,
    /// The eponymous LLVM attribute.
    MinSize,
    /// The eponymous LLVM attribute.
    MustProgress = 20,
    /// The eponymous LLVM attribute.
    Naked,
    /// The eponymous LLVM attribute.
    Nest,
    /// The eponymous LLVM attribute.
    NoAlias,
    /// The eponymous LLVM attribute.
    NoBuiltin,
    /// The eponymous LLVM attribute.
    NoCallback = 25,
    /// The eponymous LLVM attribute.
    NoCfCheck,
    /// The eponymous LLVM attribute.
    NoDivergenceSource,
    /// The eponymous LLVM attribute.
    NoDuplicate,
    /// The eponymous LLVM attribute.
    NoExt,
    /// The eponymous LLVM attribute.
    NoFree = 30,
    /// The eponymous LLVM attribute.
    NoImplicitFloat,
    /// The eponymous LLVM attribute.
    NoInline,
    /// The eponymous LLVM attribute.
    NoMerge,
    /// The eponymous LLVM attribute.
    NoProfile,
    /// The eponymous LLVM attribute.
    NoRecurse = 35,
    /// The eponymous LLVM attribute.
    NoRedZone,
    /// The eponymous LLVM attribute.
    NoReturn,
    /// The eponymous LLVM attribute.
    NoSanitizeBounds,
    /// The eponymous LLVM attribute.
    NoSanitizeCoverage,
    /// The eponymous LLVM attribute.
    NoSync = 40,
    /// The eponymous LLVM attribute.
    NoUndef,
    /// The eponymous LLVM attribute.
    NoUnwind,
    /// The eponymous LLVM attribute.
    NonLazyBind,
    /// The eponymous LLVM attribute.
    NonNull,
    /// The eponymous LLVM attribute.
    NullPointerIsValid = 45,
    /// The eponymous LLVM attribute.
    OptForFuzzing,
    /// The eponymous LLVM attribute.
    OptimizeForDebugging,
    /// The eponymous LLVM attribute.
    OptimizeForSize,
    /// The eponymous LLVM attribute.
    OptimizeNone,
    /// The eponymous LLVM attribute.
    PresplitCoroutine = 50,
    /// The eponymous LLVM attribute.
    ReadNone,
    /// The eponymous LLVM attribute.
    ReadOnly,
    /// The eponymous LLVM attribute.
    Returned,
    /// The eponymous LLVM attribute.
    ReturnsTwice,
    /// The eponymous LLVM attribute.
    SExt = 55,
    /// The eponymous LLVM attribute.
    SafeStack,
    /// The eponymous LLVM attribute.
    SanitizeAddress,
    /// The eponymous LLVM attribute.
    SanitizeHWAddress,
    /// The eponymous LLVM attribute.
    SanitizeMemTag,
    /// The eponymous LLVM attribute.
    SanitizeMemory = 60,
    /// The eponymous LLVM attribute.
    SanitizeNumericalStability,
    /// The eponymous LLVM attribute.
    SanitizeRealtime,
    /// The eponymous LLVM attribute.
    SanitizeRealtimeBlocking,
    /// The eponymous LLVM attribute.
    SanitizeThread,
    /// The eponymous LLVM attribute.
    SanitizeType = 65,
    /// The eponymous LLVM attribute.
    ShadowCallStack,
    /// The eponymous LLVM attribute.
    SkipProfile,
    /// The eponymous LLVM attribute.
    Speculatable,
    /// The eponymous LLVM attribute.
    SpeculativeLoadHardening,
    /// The eponymous LLVM attribute.
    StackProtect = 70,
    /// The eponymous LLVM attribute.
    StackProtectReq,
    /// The eponymous LLVM attribute.
    StackProtectStrong,
    /// The eponymous LLVM attribute.
    StrictFP,
    /// The eponymous LLVM attribute.
    SwiftAsync,
    /// The eponymous LLVM attribute.
    SwiftError = 75,
    /// The eponymous LLVM attribute.
    SwiftSelf,
    /// The eponymous LLVM attribute.
    WillReturn,
    /// The eponymous LLVM attribute.
    Writable,
    /// The eponymous LLVM attribute.
    WriteOnly,
    /// The eponymous LLVM attribute.
    ZExt = 80,
    /// The eponymous LLVM attribute.
    ByRef,
    /// The eponymous LLVM attribute.
    ByVal,
    /// The eponymous LLVM attribute.
    ElementType,
    /// The eponymous LLVM attribute.
    InAlloca,
    /// The eponymous LLVM attribute.
    Preallocated = 85,
    /// The eponymous LLVM attribute.
    StructRet,
    /// The eponymous LLVM attribute.
    Alignment,
    /// The eponymous LLVM attribute.
    AllocKind,
    /// The eponymous LLVM attribute.
    AllocSize,
    /// The eponymous LLVM attribute.
    Captures = 90,
    /// The eponymous LLVM attribute.
    Dereferenceable,
    /// The eponymous LLVM attribute.
    DereferenceableOrNull,
    /// The eponymous LLVM attribute.
    Memory,
    /// The eponymous LLVM attribute.
    NoFPClass,
    /// The eponymous LLVM attribute.
    StackAlignment = 95,
    /// The eponymous LLVM attribute.
    UWTable,
    /// The eponymous LLVM attribute.
    VScaleRange,
    /// The eponymous LLVM attribute.
    Range,
    /// The eponymous LLVM attribute.
    Initializes = 99,
}

impl TryFrom<&str> for Attribute {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "AlwaysInline" => Ok(Attribute::AlwaysInline),
            "Cold" => Ok(Attribute::Cold),
            "Hot" => Ok(Attribute::Hot),
            "MinSize" => Ok(Attribute::MinSize),
            "OptimizeForSize" => Ok(Attribute::OptimizeForSize),
            "NoInline" => Ok(Attribute::NoInline),
            "WillReturn" => Ok(Attribute::WillReturn),
            "NoReturn" => Ok(Attribute::NoReturn),
            "MustProgress" => Ok(Attribute::MustProgress),
            _ => Err(value.to_owned()),
        }
    }
}
