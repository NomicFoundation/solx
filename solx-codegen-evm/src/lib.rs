//!
//! EVM codegen library.
//!

#![allow(clippy::too_many_arguments)]
#![allow(clippy::upper_case_acronyms)]

pub(crate) mod codegen;
pub(crate) mod r#const;
pub(crate) mod context;
pub(crate) mod debug_config;
pub(crate) mod dependencies;
pub(crate) mod optimizer;
pub(crate) mod target_machine;

pub use self::codegen::append_metadata;
pub use self::codegen::assemble;
pub use self::codegen::attribute::Attribute as EVMAttribute;
pub use self::codegen::build::Build;
pub use self::codegen::context::address_space::AddressSpace;
pub use self::codegen::context::evmla_data::EVMLAData as ContextEVMLAData;
pub use self::codegen::context::function::intrinsics::Intrinsics;
pub use self::codegen::context::function::runtime::entry::Entry as EntryFunction;
pub use self::codegen::context::function::Function;
pub use self::codegen::context::solidity_data::SolidityData as ContextSolidityData;
pub use self::codegen::context::yul_data::YulData as ContextYulData;
pub use self::codegen::context::Context;
pub use self::codegen::initialize_target;
pub use self::codegen::instructions::arithmetic;
pub use self::codegen::instructions::bitwise;
pub use self::codegen::instructions::call;
pub use self::codegen::instructions::calldata;
pub use self::codegen::instructions::code;
pub use self::codegen::instructions::comparison;
pub use self::codegen::instructions::contract_context;
pub use self::codegen::instructions::create;
pub use self::codegen::instructions::ether_gas;
pub use self::codegen::instructions::event;
pub use self::codegen::instructions::immutable;
pub use self::codegen::instructions::math;
pub use self::codegen::instructions::memory;
pub use self::codegen::instructions::r#return;
pub use self::codegen::instructions::return_data;
pub use self::codegen::instructions::storage;
pub use self::codegen::link;
pub use self::codegen::minimal_deploy_code;
pub use self::codegen::profiler::run::Run;
pub use self::codegen::profiler::Profiler;
pub use self::codegen::warning::Warning;
pub use self::codegen::DummyLLVMWritable;
pub use self::codegen::WriteLLVM;
pub use self::codegen::IS_SIZE_FALLBACK;
pub use self::context::attribute::memory::Memory;
pub use self::context::attribute::Attribute;
pub use self::context::function::block::evmla_data::EVMLAData as FunctionBlockEVMLAData;
pub use self::context::function::block::key::Key as BlockKey;
pub use self::context::function::block::Block as FunctionBlock;
pub use self::context::function::declaration::Declaration as FunctionDeclaration;
pub use self::context::function::evmla_data::EVMLAData as FunctionEVMLAData;
pub use self::context::function::r#return::Return as FunctionReturn;
pub use self::context::pointer::Pointer;
pub use self::context::r#loop::Loop;
pub use self::context::traits::address_space::IAddressSpace;
pub use self::context::traits::evmla_data::IEVMLAData;
pub use self::context::traits::evmla_function::IEVMLAFunction;
pub use self::context::traits::solidity_data::ISolidityData;
pub use self::context::traits::yul_data::IYulData;
pub use self::context::value::Value;
pub use self::context::IContext;
pub use self::debug_config::ir_type::IRType;
pub use self::debug_config::DebugConfig;
pub use self::dependencies::Dependencies;
pub use self::optimizer::settings::size_level::SizeLevel;
pub use self::optimizer::settings::Settings as OptimizerSettings;
pub use self::optimizer::Optimizer;
pub use self::r#const::*;
pub use self::target_machine::TargetMachine;
