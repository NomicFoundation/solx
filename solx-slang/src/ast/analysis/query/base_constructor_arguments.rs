//!
//! The invocation arguments supplied to a single base constructor (pure-Slang).
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::ContractDefinition;

/// The invocation arguments supplied to one base constructor, with the contract whose scope evaluates
/// them. The arguments are evaluated in the *declaring* contract's constructor scope, binding its
/// parameters, inside that contract's constructor `sol.func`.
pub struct BaseConstructorArguments {
    /// The argument list passed to the base constructor.
    pub arguments: ArgumentsDeclaration,
    /// The contract that declares the invocation; its constructor scope evaluates the arguments.
    pub declaring_contract: ContractDefinition,
}
