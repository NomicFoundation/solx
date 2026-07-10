//!
//! ODS op-construction macros.
//!
//! `mlir_op_build!` / `mlir_op!` / `mlir_op_void!` / `mlir_region_op!` collapse the ceremony of an
//! ODS-generated op builder (the `(context, unknown_location)` head and `.build().into()` tail) so a
//! site states only the op name and its setters.
//!

/// Builds an inlined dialect op and yields it as an `Operation`, without appending.
macro_rules! mlir_op_build {
    ($context:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $operation::builder($context.melior, $context.location())
            $(.$method($($crate::IntoOds::into_ods($argument)),*))*
            .build()
            .into()
    };
}

/// Builds an inlined dialect op ([`mlir_op_build!`]), appends it to `$block`, and
/// returns its single result value. The `expect` message is derived from the op.
/// Omitting `$block` appends at the `current_block()` cursor.
macro_rules! mlir_op {
    ($context:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        mlir_op!($context, $context.current_block(), $operation $(.$method($($argument),*))*)
    };
    ($context:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $block
            .append_operation(mlir_op_build!($context, $operation $(.$method($($argument),*))*))
            .result(0)
            .expect(concat!(stringify!($operation), " produces one result"))
    };
}

/// [`mlir_op!`] for a value-less op: a statement or effect such as `sol.store`
/// or `sol.return`: appends the op ([`mlir_op_build!`]) and yields `()`.
macro_rules! mlir_op_void {
    ($context:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        mlir_op_void!($context, $context.current_block(), $operation $(.$method($($argument),*))*)
    };
    ($context:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $block.append_operation(mlir_op_build!($context, $operation $(.$method($($argument),*))*));
    };
}

/// Appends a region-bearing control-flow op (`sol.if`/`for`/`while`/`do`) and hands back each
/// region's fresh entry block for the caller to emit into and terminate. A trailing
/// `; region if condition` clause makes that region optional: it receives a block — returned as
/// `Some` — only when `condition` holds, and is left empty (`None`) otherwise, modelling an absent
/// `else`.
macro_rules! mlir_region_op {
    (
        $context:expr, $block:expr, $operation:ident
        $(.$method:ident($($argument:expr),* $(,)?))*
        ; $($region:ident),+
        $(; $optional_region:ident if $optional_condition:expr)?
        $(,)?
    ) => {{
        $(
            let $region = {
                let region = melior::ir::Region::new();
                melior::ir::RegionLike::append_block(&region, melior::ir::Block::new(&[]));
                region
            };
        )+
        $(
            let $optional_region = {
                let region = melior::ir::Region::new();
                if $optional_condition {
                    melior::ir::RegionLike::append_block(&region, melior::ir::Block::new(&[]));
                }
                region
            };
        )?
        let operation = melior::ir::BlockLike::append_operation(
            $block,
            $operation::builder($context.melior, $context.location())
                $(.$method($($crate::IntoOds::into_ods($argument)),*))*
                $(.$region($region))+
                $(.$optional_region($optional_region))?
                .build()
                .into(),
        );
        let mut regions = (0usize..).map(|index| {
            melior::ir::operation::OperationLike::region(&operation, index)
                .expect(concat!(stringify!($operation), " region index in range"))
        });
        (
            $(
                $crate::Block::from(
                    melior::ir::RegionLike::first_block(
                        &regions.next().expect(concat!("missing ", stringify!($region))),
                    )
                    .expect(concat!(stringify!($region), " has an entry block")),
                )
            ),+
            $(
                , melior::ir::RegionLike::first_block(
                    &regions.next().expect(concat!("missing ", stringify!($optional_region))),
                )
                .map($crate::Block::from)
            )?
        )
    }};
}

/// A Sol dialect attribute enum built by a `solxCreate*Attr` FFI constructor: the `#[repr(u32)]`
/// enum plus its `attribute()` builder. `From`/other impls, where present, live alongside the call.
macro_rules! sol_dialect_attribute {
    (
        $(#[$enum_meta:meta])*
        $name:ident => $ffi:path {
            $($(#[$variant_meta:meta])* $variant:ident = $value:expr),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[repr(u32)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($(#[$variant_meta])* $variant = $value),+
        }

        impl $name {
            /// Builds the corresponding Sol dialect attribute in `context`.
            pub fn attribute(self, context: &melior::Context) -> melior::ir::Attribute<'_> {
                unsafe { melior::ir::Attribute::from_raw($ffi(context.to_raw(), self as u32)) }
            }
        }
    };
}

/// A Sol comparison-predicate enum encoded as an `i64` `IntegerAttribute`: the `#[repr(i64)]` enum
/// plus its `attribute()` builder. `From`/other impls, where present, live alongside the call.
macro_rules! sol_predicate_attribute {
    (
        $(#[$enum_meta:meta])*
        $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $value:expr),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[repr(i64)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($(#[$variant_meta])* $variant = $value),+
        }

        impl $name {
            /// The `i64` `IntegerAttribute` this predicate's operand demands.
            pub fn attribute(
                self,
                context: &melior::Context,
            ) -> melior::ir::attribute::IntegerAttribute<'_> {
                melior::ir::attribute::IntegerAttribute::new(
                    melior::ir::r#type::IntegerType::new(context, solx_utils::BIT_LENGTH_X64 as u32)
                        .into(),
                    self as i64,
                )
            }
        }
    };
}

/// Coerces an op-builder setter argument to the type the ODS setter expects.
///
/// A local trait rather than `From`/`Into`: the orphan rule forbids implementing `From` for the
/// foreign melior setter types the domain conversions target, so the macros route every argument
/// through it. The reflexive impl is the identity for an argument already of the setter's type.
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

/// Declares Sol dialect op-wrapper methods on [`Value`](crate::Value), [`Place`](crate::Place), and
/// [`Block`](crate::Block) as pure data: one ODS operation per declaration.
///
/// A declaration names the receiver, the method and its typed parameters, the disposition, the
/// operation, and the builder setter chain. Every setter argument is a parameter, the receiver
/// `self`, or a closed keyword. Keywords are call-shaped, so a bare identifier is always a parameter:
/// result types `field()` / `address()` / `boolean()` / `memory()` / `calldata()` / `fixed_bytes(N)`
/// / `ptr(pointee, stack)`; the receiver-derived `self` / `self_ty` / `gep_of(elem)`; attributes
/// `int_attr` / `str_attr` / `symbol_attr` / `predicate_attr` / `ty_attr` / `count_attr`; variadic
/// operands `single` / `many` / `concat`; conditional setters `flag` / `optional_str` /
/// `optional_value`. The operation slot may be `checked(CheckedOp, UncheckedOp)`, which threads a
/// `checked: bool` selector.
///
/// Dispositions: `-> value` / `-> place` append at the `current_block()` cursor and wrap the single
/// result; `-> value nop_if_same(param)` short-circuits when the receiver already has that type; an
/// arrowless declaration is value-less and appends to the receiver block for a `Block` method, or at
/// the `current_block()` cursor for a `Value` / `Place`. A `Block` declaration listing region names
/// after `;` opens a region-bearing op and returns each region's entry block; a trailing
/// `; name if param` region is materialized only when the flag holds and is returned as an `Option`.
/// Every argument is routed through [`IntoOds`] to the setter's type.
macro_rules! sol_ops {
    () => {};

    (@ty value) => { $crate::Value<'context> };
    (@ty ty) => { $crate::Type<'context> };
    (@ty str) => { &str };
    (@ty i64) => { i64 };
    (@ty bool) => { bool };
    (@ty values) => { &[$crate::Value<'context>] };
    (@ty predicate) => { $crate::CmpPredicate };
    (@ty optional_str) => { ::core::option::Option<&str> };
    (@ty optional_value) => { ::core::option::Option<$crate::Value<'context>> };

    (@arg [$context:ident] [$receiver:tt] self) => { $receiver.inner };
    (@arg [$context:ident] [$receiver:tt] self_ty) => { $receiver.r#type() };
    (@arg [$context:ident] [$receiver:tt] gep_of($element:ident)) => {
        $receiver.r#type().gep_result_type($element)
    };
    (@arg [$context:ident] [$receiver:tt] field()) => {
        $crate::Type::unsigned($context.melior, solx_utils::BIT_LENGTH_FIELD)
    };
    (@arg [$context:ident] [$receiver:tt] address()) => {
        $crate::Type::address($context.melior, false)
    };
    (@arg [$context:ident] [$receiver:tt] boolean()) => {
        $crate::Type::signless($context.melior, solx_utils::BIT_LENGTH_BOOLEAN)
    };
    (@arg [$context:ident] [$receiver:tt] memory()) => {
        $crate::Type::string($context.melior, solx_utils::DataLocation::Memory)
    };
    (@arg [$context:ident] [$receiver:tt] calldata()) => {
        $crate::Type::string($context.melior, solx_utils::DataLocation::CallData)
    };
    (@arg [$context:ident] [$receiver:tt] fixed_bytes($width:literal)) => {
        $crate::Type::fixed_bytes($context.melior, $width)
    };
    (@arg [$context:ident] [$receiver:tt] ptr($pointee:ident, stack)) => {
        $crate::Type::pointer($context.melior, $pointee, solx_utils::DataLocation::Stack)
    };
    (@arg [$context:ident] [$receiver:tt] int_attr($value:ident, $result_type:ident)) => {
        ::melior::ir::Attribute::from(::melior::ir::attribute::IntegerAttribute::new(
            $result_type.into_mlir(),
            $value,
        ))
    };
    (@arg [$context:ident] [$receiver:tt] str_attr($text:ident)) => {
        ::melior::ir::attribute::StringAttribute::new($context.melior, $text)
    };
    (@arg [$context:ident] [$receiver:tt] symbol_attr($name:ident)) => {
        ::melior::ir::attribute::FlatSymbolRefAttribute::new($context.melior, $name)
    };
    (@arg [$context:ident] [$receiver:tt] predicate_attr($predicate:ident)) => {
        ::melior::ir::Attribute::from($predicate.attribute($context.melior))
    };
    (@arg [$context:ident] [$receiver:tt] ty_attr($($inner:tt)*)) => {
        ::melior::ir::attribute::TypeAttribute::new(
            sol_ops!(@arg [$context] [$receiver] $($inner)*).into_mlir(),
        )
    };
    (@arg [$context:ident] [$receiver:tt] count_attr($topics:ident)) => {
        ::melior::ir::attribute::IntegerAttribute::new(
            ::melior::ir::r#type::IntegerType::new($context.melior, 8).into(),
            i8::try_from($topics.len())
                .expect("EVM events carry at most four indexed arguments")
                .into(),
        )
    };
    (@arg [$context:ident] [$receiver:tt] $parameter:ident) => { $parameter };

    (@operands $iterator:expr) => {
        &$iterator
            .map(|operand| $crate::IntoOds::into_ods(*operand))
            .collect::<::std::vec::Vec<_>>()
    };

    (@chain $builder:ident [$context:ident] [$receiver:tt]) => { $builder };
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (flag($flag:ident)) $($rest:tt)*) => {{
        let $builder = if $flag {
            $builder.$setter(::melior::ir::Attribute::unit($context.melior))
        } else {
            $builder
        };
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (optional_str($text:ident)) $($rest:tt)*) => {{
        let $builder = if let ::core::option::Option::Some(__text) = $text {
            $builder.$setter(sol_ops!(@arg [$context] [$receiver] str_attr(__text)))
        } else {
            $builder
        };
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (optional_value($operand:ident)) $($rest:tt)*) => {{
        let $builder = if let ::core::option::Option::Some(__operand) = $operand {
            $builder.$setter($crate::IntoOds::into_ods(__operand))
        } else {
            $builder
        };
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (many($operands:ident)) $($rest:tt)*) => {{
        let $builder = $builder.$setter(sol_ops!(@operands $operands.iter()));
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (concat($head:ident, $tail:ident)) $($rest:tt)*) => {{
        let $builder = $builder.$setter(sol_ops!(@operands $head.iter().chain($tail.iter())));
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident (single($operand:ident)) $($rest:tt)*) => {{
        let $builder = $builder.$setter(&[$crate::IntoOds::into_ods($operand)]);
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};
    (@chain $builder:ident [$context:ident] [$receiver:tt] .$setter:ident ($($argument:tt)*) $($rest:tt)*) => {{
        let $builder = $builder.$setter($crate::IntoOds::into_ods(sol_ops!(@arg [$context] [$receiver] $($argument)*)));
        sol_ops!(@chain $builder [$context] [$receiver] $($rest)*)
    }};

    (@build [$context:ident] [$receiver:tt] $operation:ident $($chain:tt)*) => {
        {
            let builder = $operation::builder($context.melior, $context.location());
            sol_ops!(@chain builder [$context] [$receiver] $($chain)*)
        }
        .build()
        .into()
    };

    (@disp_ty value) => { $crate::Value<'context> };
    (@disp_ty place) => { $crate::Place<'context> };

    (@region_tuple $first:ident, $second:ident) => {
        ($crate::Block<'context>, $crate::Block<'context>)
    };
    (@region_tuple $first:ident, $second:ident, $third:ident) => {
        ($crate::Block<'context>, $crate::Block<'context>, $crate::Block<'context>)
    };
    (@region_tuple $region:ident ; $optional_region:ident if $flag:ident) => {
        ($crate::Block<'context>, ::core::option::Option<$crate::Block<'context>>)
    };

    (@one_result [$context:ident] $operation:expr, $message:expr) => {
        $context
            .current_block()
            .append_operation($operation)
            .result(0)
            .expect($message)
    };
    (@emit value [$context:ident] $operation:expr, $message:expr) => {
        $crate::Value::from(sol_ops!(@one_result [$context] $operation, $message))
    };
    (@emit place [$context:ident] $operation:expr, $message:expr) => {
        $crate::Place::from(sol_ops!(@one_result [$context] $operation, $message))
    };

    (
        $receiver:ident :: $method:ident (self $(, $argument:ident : $kind:ident)* $(,)?)
        -> value { checked($checked_op:ident, $unchecked_op:ident) $($chain:tt)* }
        $($rest:tt)*
    ) => {
        impl<'context> $receiver<'context> {
            pub fn $method(
                self,
                $($argument: sol_ops!(@ty $kind),)*
                checked: bool,
                context: &$crate::Context<'context>,
            ) -> $crate::Value<'context> {
                let receiver = self;
                let operation = if checked {
                    sol_ops!(@build [context] [receiver] $checked_op $($chain)*)
                } else {
                    sol_ops!(@build [context] [receiver] $unchecked_op $($chain)*)
                };
                sol_ops!(@emit value [context] operation, "checked arithmetic op produces one result")
            }
        }
        sol_ops!($($rest)*);
    };

    (
        $receiver:ident :: $method:ident (self $(, $argument:ident : $kind:ident)* $(,)?)
        -> $disposition:ident $(nop_if_same($same:ident))? { $operation:ident $($chain:tt)* }
        $($rest:tt)*
    ) => {
        impl<'context> $receiver<'context> {
            pub fn $method(
                self,
                $($argument: sol_ops!(@ty $kind),)*
                context: &$crate::Context<'context>,
            ) -> sol_ops!(@disp_ty $disposition) {
                let receiver = self;
                $(if receiver.r#type() == $same {
                    return receiver.into();
                })?
                sol_ops!(@emit $disposition [context]
                    sol_ops!(@build [context] [receiver] $operation $($chain)*),
                    concat!(stringify!($operation), " produces one result"))
            }
        }
        sol_ops!($($rest)*);
    };

    (
        $receiver:ident :: $method:ident ($($argument:ident : $kind:ident),* $(,)?)
        -> $disposition:ident { $operation:ident $($chain:tt)* }
        $($rest:tt)*
    ) => {
        impl<'context> $receiver<'context> {
            pub fn $method(
                $($argument: sol_ops!(@ty $kind),)*
                context: &$crate::Context<'context>,
            ) -> sol_ops!(@disp_ty $disposition) {
                sol_ops!(@emit $disposition [context]
                    sol_ops!(@build [context] [()] $operation $($chain)*),
                    concat!(stringify!($operation), " produces one result"))
            }
        }
        sol_ops!($($rest)*);
    };

    (
        Block :: $method:ident (self $(, $argument:ident : $kind:ident)* $(,)?)
        { $operation:ident $(.$setter:ident($($source:tt)*))* ; $($regions:tt)+ }
        $($rest:tt)*
    ) => {
        impl<'context> Block<'context> {
            pub fn $method(
                self,
                $($argument: sol_ops!(@ty $kind),)*
                context: &$crate::Context<'context>,
            ) -> sol_ops!(@region_tuple $($regions)+) {
                let receiver = self;
                mlir_region_op!(
                    context,
                    &receiver.inner,
                    $operation $(.$setter(sol_ops!(@arg [context] [receiver] $($source)*)))*
                    ; $($regions)+
                )
            }
        }
        sol_ops!($($rest)*);
    };

    (
        Block :: $method:ident (self $(, $argument:ident : $kind:ident)* $(,)?)
        { $operation:ident $($chain:tt)* }
        $($rest:tt)*
    ) => {
        impl<'context> Block<'context> {
            pub fn $method(
                self,
                $($argument: sol_ops!(@ty $kind),)*
                context: &$crate::Context<'context>,
            ) {
                let receiver = self;
                receiver
                    .inner
                    .append_operation(sol_ops!(@build [context] [receiver] $operation $($chain)*));
            }
        }
        sol_ops!($($rest)*);
    };

    (
        $receiver:ident :: $method:ident (self $(, $argument:ident : $kind:ident)* $(,)?)
        { $operation:ident $($chain:tt)* }
        $($rest:tt)*
    ) => {
        impl<'context> $receiver<'context> {
            pub fn $method(
                self,
                $($argument: sol_ops!(@ty $kind),)*
                context: &$crate::Context<'context>,
            ) {
                let receiver = self;
                context
                    .current_block()
                    .append_operation(sol_ops!(@build [context] [receiver] $operation $($chain)*));
            }
        }
        sol_ops!($($rest)*);
    };

    (
        $receiver:ident :: $method:ident ($($argument:ident : $kind:ident),* $(,)?)
        { $operation:ident $($chain:tt)* }
        $($rest:tt)*
    ) => {
        impl<'context> $receiver<'context> {
            pub fn $method(
                $($argument: sol_ops!(@ty $kind),)*
                context: &$crate::Context<'context>,
            ) {
                context
                    .current_block()
                    .append_operation(sol_ops!(@build [context] [()] $operation $($chain)*));
            }
        }
        sol_ops!($($rest)*);
    };
}
