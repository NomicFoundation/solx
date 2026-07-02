//!
//! Sol and Yul dialect attribute enums for MLIR code generation.
//!

use crate::ffi;

sol_dialect_attribute! {
    /// Sol dialect contract kind.
    ContractKind => ffi::solxCreateContractKindAttr {
        /// Interface contract.
        Interface = 0,
        /// Regular contract.
        Contract = 1,
        /// Library contract.
        Library = 2,
    }
}

sol_dialect_attribute! {
    /// Sol dialect function kind. Regular functions do not carry a kind attribute.
    FunctionKind => ffi::solxCreateFunctionKindAttr {
        /// Constructor function.
        Constructor = 0,
        /// Fallback function.
        Fallback = 1,
        /// Receive function.
        Receive = 2,
    }
}

sol_dialect_attribute! {
    /// Sol dialect state mutability.
    StateMutability => ffi::solxCreateStateMutabilityAttr {
        /// No state reads or writes.
        Pure = 0,
        /// Reads state, no writes.
        View = 1,
        /// Reads and writes state, no ether.
        NonPayable = 2,
        /// Can receive ether.
        Payable = 3,
    }
}

sol_predicate_attribute! {
    /// Sol dialect `sol.cmp` predicate values. Signedness is carried by the operand type, not the
    /// predicate.
    CmpPredicate {
        /// Equal.
        Eq = 0,
        /// Not equal.
        Ne = 1,
        /// Less than.
        Lt = 2,
        /// Less than or equal.
        Le = 3,
        /// Greater than.
        Gt = 4,
        /// Greater than or equal.
        Ge = 5,
    }
}

sol_predicate_attribute! {
    /// Yul dialect `yul.cmp` predicate values.
    YulCmpPredicate {
        /// Equal (`eq`).
        Eq = 0,
        /// Not equal (`ne`).
        Ne = 1,
        /// Unsigned less than, `ult`, Yul `lt`.
        Ult = 2,
        /// Unsigned less than or equal (`ule`).
        Ule = 3,
        /// Unsigned greater than, `ugt`, Yul `gt`.
        Ugt = 4,
        /// Unsigned greater than or equal (`uge`).
        Uge = 5,
        /// Signed less than, `slt`, Yul `slt`.
        Slt = 6,
        /// Signed less than or equal (`sle`).
        Sle = 7,
        /// Signed greater than, `sgt`, Yul `sgt`.
        Sgt = 8,
        /// Signed greater than or equal (`sge`).
        Sge = 9,
    }
}
