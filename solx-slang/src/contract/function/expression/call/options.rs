//!
//! The gas, value and salt a call forwards, from the options decorating it.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::CallOptions;

use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

/// The gas, value and salt a call forwards. An option the call omits stays absent and materializes
/// its default at the op: the named options evaluate after the callee, the defaults after the
/// arguments.
#[derive(Default)]
pub struct Options<'context> {
    /// The gas the call forwards.
    gas: Option<Value<'context>>,
    /// The value the call transfers, never named on a delegate or static call.
    amount: Option<Value<'context>>,
    /// The salt a creation derives its address from, named only by `new`.
    salt: Option<Value<'context>>,
}

impl<'context> Options<'context> {
    /// Emits each option the call names, in source order; an undecorated call forwards defaults.
    pub fn new(options: Option<&CallOptions>, scope: &mut FunctionScope<'_, '_, 'context>) -> Self {
        let Some(options) = options else {
            return Self::default();
        };
        let field = MlirType::field(scope.melior);
        let mut forwarded = Self::default();
        for option in options.iter() {
            match option.name().resolve_to_built_in() {
                Some(BuiltIn::CallOptionGas) => {
                    forwarded.gas = Some(scope.converted(&option.value(), field));
                }
                Some(BuiltIn::CallOptionValue) => {
                    forwarded.amount = Some(scope.converted(&option.value(), field));
                }
                Some(BuiltIn::CallOptionSalt) => {
                    let salt = scope.converted(
                        &option.value(),
                        MlirType::fixed_bytes(scope.melior, solx_utils::BYTE_LENGTH_FIELD),
                    );
                    forwarded.salt = Some(salt.bytes_cast(field, scope));
                }
                _ => unreachable!("slang admits gas, value and salt as the only call options"),
            }
        }
        forwarded
    }

    /// The gas operand: all the caller has left where the call names none.
    pub fn gas(&self, scope: &mut FunctionScope<'_, '_, 'context>) -> Value<'context> {
        self.gas.unwrap_or_else(|| Value::gas_left(scope))
    }

    /// The value operand, zero where the call transfers none.
    pub fn amount(&self, scope: &mut FunctionScope<'_, '_, 'context>) -> Value<'context> {
        self.amount
            .unwrap_or_else(|| Value::zero(MlirType::field(scope.melior), scope))
    }

    /// The salt operand, absent where a creation derives its address through `CREATE`.
    pub fn salt(&self) -> Option<Value<'context>> {
        self.salt
    }
}
