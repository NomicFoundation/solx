//!
//! ODS op-construction macros.
//!
//! `mlir_op_build!` / `mlir_op!` / `mlir_op_void!` / `mlir_region_op!` collapse the ceremony of an
//! ODS-generated op builder (the `(context, unknown_location)` head and `.build().into()` tail) so a
//! site states only the op name and its setters. Ops with optional setters applied conditionally
//! (`Encode`'s `selector`, `New`'s `salt`, `Emit`'s `signature`), multiple results (`Decode`), or
//! `sol.try`'s conditional catch regions are built by hand.
//!

/// Coerces one op-builder setter argument to the type the ODS setter expects (identity for most types).
pub trait IntoOds<T> {
    /// Converts `self` into the setter's argument type.
    fn into_ods(self) -> T;
}

impl<T> IntoOds<T> for T {
    fn into_ods(self) -> T {
        self
    }
}

impl<'slice, T, const N: usize> IntoOds<&'slice [T]> for &'slice [T; N] {
    fn into_ods(self) -> &'slice [T] {
        self
    }
}

/// Builds an inlined dialect op and yields it as an `Operation`, without appending.
#[macro_export]
macro_rules! mlir_op_build {
    ($builder:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $operation::builder($builder.context, $builder.unknown_location)
            $(.$method($($crate::IntoOds::into_ods($argument)),*))*
            .build()
            .into()
    };
}

/// Builds an inlined dialect op ([`mlir_op_build!`]), appends it to `$block`, and
/// returns its single result value. The `expect` message is derived from the op.
#[macro_export]
macro_rules! mlir_op {
    ($builder:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $block
            .append_operation(mlir_op_build!($builder, $operation $(.$method($($argument),*))*))
            .result(0)
            .expect(concat!(stringify!($operation), " produces one result"))
            .into()
    };
}

/// [`mlir_op!`] for a value-less op — a statement / effect such as `sol.transfer`
/// or `sol.log`: appends the op ([`mlir_op_build!`]) and yields `()`.
#[macro_export]
macro_rules! mlir_op_void {
    ($builder:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $block.append_operation(mlir_op_build!($builder, $operation $(.$method($($argument),*))*));
    };
}

/// Appends a region-bearing control-flow op (`sol.if`/`for`/`while`/`do`) and yields each region's
/// fresh entry block (named after the `;`) for the caller to emit into and terminate.
#[macro_export]
macro_rules! mlir_region_op {
    (
        $builder:expr, $block:expr, $operation:ident
        $(.$method:ident($($argument:expr),* $(,)?))*
        ; $($region:ident),+ $(,)?
    ) => {{
        $(
            let $region = {
                let region = melior::ir::Region::new();
                melior::ir::RegionLike::append_block(&region, melior::ir::Block::new(&[]));
                region
            };
        )+
        let operation = melior::ir::BlockLike::append_operation(
            $block,
            $operation::builder($builder.context, $builder.unknown_location)
                $(.$method($($crate::IntoOds::into_ods($argument)),*))*
                $(.$region($region))+
                .build()
                .into(),
        );
        let mut regions = (0usize..).map(|index| {
            melior::ir::RegionLike::first_block(
                &melior::ir::operation::OperationLike::region(&operation, index)
                    .expect(concat!(stringify!($operation), " region index in range")),
            )
            .expect(concat!(stringify!($operation), " region has an entry block"))
        });
        ($(
            regions.next().expect(concat!("missing ", stringify!($region)))
        ),+)
    }};
}
