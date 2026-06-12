//!
//! Crate-wide macros for inlined Sol dialect op construction.
//!
//! Set-B ops (§2b) are constructed inline via `<Op>::builder(…)`; these macros
//! collapse the surrounding ceremony — the `(context, unknown_location)` head
//! and the `.build().into()` tail — so only the op name and its field setters
//! stay on screen. They do NOT add `emit_sol_*` Builder methods (that is Set A):
//! the expansion is still an inline ODS construction.
//!
//! [`sol_op_build`] is the core (build the `Operation`); [`sol_op`] and
//! [`sol_op_void`] layer the append (and single-result extraction) on top of it,
//! so the builder construction is written exactly once:
//! - [`sol_op_build`] — yield the `Operation`, do not append (for a `match` arm
//!   / closure that hands the op to a shared append site);
//! - [`sol_op`] — append to a block and return the single result value;
//! - [`sol_op_void`] — append a value-less effect op.
//!
//! Ops with `operand_segment_sizes` (`Encode`, `New`, `Emit`) or multiple
//! results (`Decode`) are built by hand — their construction is not a fixed
//! method chain.
//!

/// Builds an inlined dialect op and yields it as an `Operation`, without
/// appending. The op-builder method chain is written inline after the op name.
macro_rules! sol_op_build {
    ($builder:expr, $op:ident $(.$method:ident($($arg:expr),* $(,)?))+) => {
        $op::builder($builder.context, $builder.unknown_location)
            $(.$method($($arg),*))+
            .build()
            .into()
    };
}

/// Builds an inlined dialect op ([`sol_op_build!`]), appends it to `$block`, and
/// returns its single result value. The `expect` message is derived from the op.
macro_rules! sol_op {
    ($builder:expr, $block:expr, $op:ident $(.$method:ident($($arg:expr),* $(,)?))+) => {
        $block
            .append_operation(sol_op_build!($builder, $op $(.$method($($arg),*))+))
            .result(0)
            .expect(concat!(stringify!($op), " produces one result"))
            .into()
    };
}

/// [`sol_op!`] for a value-less op — a statement / effect such as `sol.transfer`
/// or `sol.log`: appends the op ([`sol_op_build!`]) and yields `()`.
macro_rules! sol_op_void {
    ($builder:expr, $block:expr, $op:ident $(.$method:ident($($arg:expr),* $(,)?))+) => {
        $block.append_operation(sol_op_build!($builder, $op $(.$method($($arg),*))+));
    };
}
