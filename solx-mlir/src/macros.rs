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
//! - [`sol_op_void`] — append a value-less effect op.
//!
//! Ops with `operand_segment_sizes` (`Encode`, `New`, `Emit`) or multiple
//! results (`Decode`) are built by hand — their construction is not a fixed
//! method chain.
//!
//! [`Value`]: crate::Value
//! [`Type`]: crate::Type
//!

/// Builds an inlined dialect op and yields it as an `Operation`, without
/// appending. The op-builder method chain is written inline after the op name.
/// The setter repetition is `*` (not `+`): a field-less op (`sol.break`,
/// `sol.continue`) is written as the bare op name with no setters.
#[macro_export]
macro_rules! sol_op_build {
    ($builder:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $operation::builder($builder.context, $builder.unknown_location)
            $(.$method($($argument),*))*
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
