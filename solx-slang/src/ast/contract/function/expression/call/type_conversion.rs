//!
//! Solidity type conversion classification and dispatch.
//!

use melior::ir::Type;
use melior::ir::TypeLike;
use melior::ir::r#type::IntegerType;
use slang_solidity::backend::ir::ast::Type as SlangType;

/// Classification of Solidity type conversions.
///
/// Used for both explicit conversions (`uint256(x)`, `address(x)`, `bool(x)`)
/// and implicit operand widening in arithmetic, assignment, and comparison.
pub enum TypeConversion<'context> {
    /// `bool(x)` — comparison against zero, not bit-truncation.
    Bool,
    /// `address(x)` / `payable(x)` — `sol.cast` to ui160 then `sol.address_cast`.
    Address,
    /// Integer type cast — `sol.cast` to target.
    Cast(Type<'context>),
}

impl<'context> TypeConversion<'context> {
    /// Maps a Slang semantic type to an MLIR type.
    pub fn resolve_slang_type(
        slang_type: &SlangType,
        context: &'context melior::Context,
        builder: &solx_mlir::Builder<'context>,
    ) -> Type<'context> {
        match slang_type {
            SlangType::Integer(integer_type) => {
                let bits = integer_type.bits();
                if integer_type.signed() {
                    Type::from(IntegerType::signed(context, bits))
                } else {
                    Type::from(IntegerType::unsigned(context, bits))
                }
            }
            SlangType::Boolean(_) => Type::from(IntegerType::new(
                context,
                solx_utils::BIT_LENGTH_BOOLEAN as u32,
            )),
            SlangType::Address(_) => builder.get_type(solx_mlir::Builder::SOL_ADDRESS),
            SlangType::Literal(_) => builder.get_type(solx_mlir::Builder::UI256),
            _ => unimplemented!("unsupported Slang type"),
        }
    }

    /// Classifies a target type into the appropriate conversion variant.
    pub fn from_target_type(
        target_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Self {
        if target_type == builder.get_type(solx_mlir::Builder::I1) {
            Self::Bool
        } else if target_type == builder.get_type(solx_mlir::Builder::SOL_ADDRESS) {
            Self::Address
        } else {
            Self::Cast(target_type)
        }
    }

    /// Emits the conversion, returning the cast value.
    pub fn emit(
        self,
        value: melior::ir::Value<'context, '_>,
        builder: &solx_mlir::Builder<'context>,
        block: &melior::ir::BlockRef<'context, '_>,
    ) -> melior::ir::Value<'context, '_> {
        match self {
            Self::Bool => {
                let zero = builder.emit_sol_constant(0, value.r#type(), block);
                builder.emit_sol_cmp(value, zero, solx_mlir::CmpPredicate::Ne, block)
            }
            Self::Address => {
                let address_type = builder.get_type(solx_mlir::Builder::SOL_ADDRESS);
                let truncated = if melior::ir::r#type::IntegerType::try_from(value.r#type()).is_ok()
                {
                    let ui160 = builder.get_type(solx_mlir::Builder::UI160);
                    builder.emit_sol_cast(value, ui160, block)
                } else {
                    value
                };
                builder.emit_sol_address_cast(truncated, address_type, block)
            }
            Self::Cast(target_type) => builder.emit_sol_cast(value, target_type, block),
        }
    }
}
