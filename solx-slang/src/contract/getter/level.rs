//!
//! One key or index level a getter peels off its state variable's declared type.
//!

use solx_mlir::Context;
use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

/// One key or index level of the getter's storage walk, consuming one getter parameter.
pub enum Level<'context> {
    /// A mapping key step; the entry type follows the `Sol_GepOp` storage-pointer rule, where a
    /// reference-typed entry is its own address.
    Mapping {
        /// The address type the `sol.map` step yields.
        entry_type: MlirType<'context>,
    },
    /// An array index step; the element type comes off the place. The index plain-reverts out of
    /// bounds rather than raising `Panic(0x32)`.
    Array,
}

impl<'context> Level<'context> {
    /// Steps the place by one getter parameter.
    pub fn step(
        &self,
        place: Place<'context>,
        argument: Value<'context>,
        context: &Context<'context>,
    ) -> Place<'context> {
        match self {
            Self::Mapping { entry_type } => place.map(argument, *entry_type, context),
            Self::Array => {
                let element_type = place.r#type().element_type(0);
                place.gep_no_panic_bounds(argument, element_type, context)
            }
        }
    }
}
