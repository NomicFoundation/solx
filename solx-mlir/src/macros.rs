//!
//! ODS op-construction macros.
//!
//! [`sol_op_build`] / [`sol_op`] / [`sol_op_void`] collapse the ceremony of an
//! ODS-generated op builder — the `(context, unknown_location)` head and the
//! `.build().into()` tail — so a construction site states only the op name and
//! its field setters. They re-spell nothing: the expansion *is* the inline ODS
//! construction, used inside the entity ([`Value`]/[`Type`]) or frontend node
//! that owns the op.
//!
//! [`sol_op_build`] is the core (build the `Operation`); [`sol_op`] and
//! [`sol_op_void`] layer the append (and single-result extraction) on top, so
//! the builder chain is written exactly once:
//! - [`sol_op_build`] — yield the `Operation`, do not append;
//! - [`sol_op`] — append to a block and return the single result value;
//! - [`sol_op_void`] — append a value-less effect op;
//! - [`sol_region_op`] — append a region-bearing control-flow op (`sol.if` /
//!   `sol.for` / `sol.while` / `sol.do`), each region a fresh empty single
//!   block, and yield those entry blocks for the caller to emit into. This is
//!   the one home of the region-and-block plumbing the three `sol.if` sites
//!   (`if`, `&&`/`||`, `?:`) and the loops would otherwise each repeat.
//!
//! Ops with `operand_segment_sizes` (`Encode`, `New`, `Emit`) or multiple
//! results (`Decode`) are built by hand — their construction is not a fixed
//! method chain. `sol.try` likewise: its catch regions are conditionally
//! present and carry typed block arguments, so it is built by hand in its one
//! owning node rather than through [`sol_region_op`].
//!
//! [`Value`]: crate::Value
//! [`Type`]: crate::Type
//!

/// Coerces one op-builder setter argument to the type the ODS setter expects.
/// Identity for everything except the `Value` / `Type` entities (which home
/// their own impls in `context/`) and the array-to-slice passthrough the
/// operand setters need, so a `sol_op*` site writes `AstType::field(builder)`
/// instead of `…field(builder).into_mlir()`.
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

/// Builds an inlined dialect op and yields it as an `Operation`, without
/// appending. The op-builder method chain is written inline after the op name.
/// The setter repetition is `*` (not `+`): a field-less op (`sol.break`,
/// `sol.continue`) is written as the bare op name with no setters.
#[macro_export]
macro_rules! sol_op_build {
    ($builder:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $operation::builder($builder.context, $builder.unknown_location)
            $(.$method($($crate::IntoOds::into_ods($argument)),*))*
            .build()
            .into()
    };
}

/// Builds an inlined dialect op ([`sol_op_build!`]), appends it to `$block`, and
/// returns its single result value. The `expect` message is derived from the op.
#[macro_export]
macro_rules! sol_op {
    ($builder:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $block
            .append_operation(sol_op_build!($builder, $operation $(.$method($($argument),*))*))
            .result(0)
            .expect(concat!(stringify!($operation), " produces one result"))
            .into()
    };
}

/// [`sol_op!`] for a value-less op — a statement / effect such as `sol.transfer`
/// or `sol.log`: appends the op ([`sol_op_build!`]) and yields `()`.
#[macro_export]
macro_rules! sol_op_void {
    ($builder:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $block.append_operation(sol_op_build!($builder, $operation $(.$method($($argument),*))*));
    };
}

/// Appends a region-bearing control-flow op and yields its region entry blocks
/// in declaration order. Each region named after the `;` is created as a fresh
/// single empty block and fed to the op's like-named region setter; the value
/// setters before the `;` (e.g. `sol.if`'s `.cond(...)`) are written as for
/// [`sol_op!`]. The yielded tuple is the entry block per region — the caller
/// emits each branch body into it and terminates it (`sol.yield` /
/// `sol.condition`).
///
/// Every covered op (`sol.if`/`for`/`while`/`do`) has empty, unconditional
/// regions, so the plumbing is uniform; `sol.try`'s arg-bearing conditional
/// catch regions are the exception built by hand (see the module doc).
#[macro_export]
macro_rules! sol_region_op {
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
