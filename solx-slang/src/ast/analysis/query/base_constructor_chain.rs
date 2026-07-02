//!
//! Base-constructor chaining over the C3 linearisation (pure-Slang).
//!
//! An inheritance chain emits one `sol.func` per constructor: the most-derived constructor
//! (`kind = #Constructor`) and one plain internal `sol.func` per other constructor in the
//! linearisation. Each such function `sol.call`s the *next* constructor in the chain, passing the
//! invocation arguments supplied for that base: whether written as an inline `is Base(args)` on the
//! contract header or as a `Base(args)` invocation in the constructor's modifier list. This module
//! resolves both facts: which arguments a base constructor receives, and which constructor comes next.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;

use crate::ast::analysis::query::base_constructor_arguments::BaseConstructorArguments;
use crate::ast::analysis::query::match_linearised_base::MatchLinearisedBase;

/// Resolves the base-constructor call chain a contract's construction emits.
pub trait BaseConstructorChain {
    /// Collects, for the whole C3 hierarchy, the invocation arguments supplied to each base
    /// constructor: keyed by the base *contract*'s node id, valued by the argument expressions and
    /// the contract whose scope evaluates them (the contract declaring the `is Base(args)` / `Base(args)`).
    ///
    /// Solidity forbids supplying a base constructor's arguments twice, so a single source wins per base.
    fn base_constructor_arguments(
        &self,
        mro: &[ContractDefinition],
    ) -> HashMap<NodeId, BaseConstructorArguments>;

    /// The constructor that comes next in the chain after `contract`: the first contract strictly
    /// after it in the C3 linearisation, most-derived first, that declares a constructor, or `None`
    /// when the rest of the chain contributes none.
    fn next_constructor_contract(
        &self,
        contract: &ContractDefinition,
        mro: &[ContractDefinition],
    ) -> Option<ContractDefinition>;
}

impl BaseConstructorChain for ContractDefinition {
    fn base_constructor_arguments(
        &self,
        mro: &[ContractDefinition],
    ) -> HashMap<NodeId, BaseConstructorArguments> {
        let mut collected: HashMap<NodeId, BaseConstructorArguments> = HashMap::new();
        for declaring_contract in mro.iter() {
            if let Some(constructor) = declaring_contract.constructor() {
                for invocation in constructor.modifier_invocations().iter() {
                    let Some(arguments) = invocation.arguments() else {
                        continue;
                    };
                    let Some(base_contract) = invocation.name().match_linearised_base(mro) else {
                        continue;
                    };
                    collected.insert(
                        base_contract.node_id(),
                        BaseConstructorArguments {
                            arguments,
                            declaring_contract: declaring_contract.clone(),
                        },
                    );
                }
            }
            for inheritance in declaring_contract.inheritance_types().iter() {
                let Some(arguments) = inheritance.arguments() else {
                    continue;
                };
                let Some(base_contract) = inheritance.type_name().match_linearised_base(mro) else {
                    continue;
                };
                collected.insert(
                    base_contract.node_id(),
                    BaseConstructorArguments {
                        arguments,
                        declaring_contract: declaring_contract.clone(),
                    },
                );
            }
        }
        collected
    }

    fn next_constructor_contract(
        &self,
        contract: &ContractDefinition,
        mro: &[ContractDefinition],
    ) -> Option<ContractDefinition> {
        let position = mro
            .iter()
            .position(|candidate| candidate.node_id() == contract.node_id())?;
        mro.iter()
            .skip(position + 1)
            .find(|candidate| candidate.constructor().is_some())
            .cloned()
    }
}
