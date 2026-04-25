//!
//! Solidity type conversion classification and dispatch.
//!

use melior::ir::Type;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use slang_solidity::backend::ir::ast::FunctionDefinition;
use slang_solidity::backend::ir::ast::StateVariableDefinition;
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
        builder: &solx_mlir::Builder<'context>,
    ) -> Type<'context> {
        match slang_type {
            SlangType::Integer(integer_type) => {
                let bits = integer_type.bits();
                if integer_type.signed() {
                    Type::from(IntegerType::signed(builder.context, bits))
                } else {
                    Type::from(IntegerType::unsigned(builder.context, bits))
                }
            }
            SlangType::Boolean(_) => Type::from(IntegerType::new(
                builder.context,
                solx_utils::BIT_LENGTH_BOOLEAN as u32,
            )),
            SlangType::Address(_) => builder.types.sol_address,
            SlangType::Literal(_) => builder.types.ui256,
            _ => unimplemented!("unsupported Slang type"),
        }
    }

    /// Classifies a target type into the appropriate conversion variant.
    pub fn from_target_type(
        target_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Self {
        if target_type == builder.types.i1 {
            Self::Bool
        } else if target_type == builder.types.sol_address {
            Self::Address
        } else {
            Self::Cast(target_type)
        }
    }

    /// Resolves the declared Solidity type of a state variable to an MLIR type.
    pub fn resolve_state_variable_type(
        state_variable: &StateVariableDefinition,
        builder: &solx_mlir::Builder<'context>,
    ) -> anyhow::Result<Type<'context>> {
        let name = state_variable.name().name();
        let slang_type = state_variable
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("unresolved type for state variable: {name}"))?;
        Ok(Self::resolve_slang_type(&slang_type, builder))
    }

    /// Resolves a function's parameter and return types from Slang AST to MLIR.
    pub fn resolve_function_types(
        function: &FunctionDefinition,
        builder: &solx_mlir::Builder<'context>,
    ) -> (Vec<Type<'context>>, Vec<Type<'context>>) {
        let parameter_types = function
            .parameters()
            .iter()
            .map(|parameter| {
                Self::resolve_slang_type(
                    &parameter
                        .get_type()
                        .expect("parameter type resolved by semantic analysis"),
                    builder,
                )
            })
            .collect();
        let return_types = function
            .returns()
            .map(|returns| {
                returns
                    .iter()
                    .map(|parameter| {
                        Self::resolve_slang_type(
                            &parameter
                                .get_type()
                                .expect("return type resolved by semantic analysis"),
                            builder,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        (parameter_types, return_types)
    }

    /// Emits the conversion, returning the cast value.
    pub fn emit<'block>(
        self,
        value: melior::ir::Value<'context, 'block>,
        builder: &solx_mlir::Builder<'context>,
        block: &melior::ir::BlockRef<'context, 'block>,
    ) -> melior::ir::Value<'context, 'block>
    where
        'context: 'block,
    {
        match self {
            Self::Bool => {
                let zero = builder.emit_sol_constant(0, value.r#type(), block);
                builder.emit_sol_cmp(value, zero, solx_mlir::CmpPredicate::Ne, block)
            }
            Self::Address => {
                let address_type = builder.types.sol_address;
                let truncated = if melior::ir::r#type::IntegerType::try_from(value.r#type()).is_ok()
                {
                    let ui160 = builder.types.ui160;
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
