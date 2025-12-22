//!
//! The LLVM IR generator function.
//!

pub mod intrinsics;
pub mod runtime;

use std::collections::HashMap;

use crate::codegen::attribute::Attribute as StringAttribute;
use crate::codegen::context::address_space::AddressSpace;
use crate::context::attribute::Attribute;
use crate::context::function::block::key::Key as BlockKey;
use crate::context::function::block::Block;
use crate::context::function::declaration::Declaration as FunctionDeclaration;
use crate::context::function::evmla_data::EVMLAData as FunctionEVMLAData;
use crate::context::function::r#return::Return as FunctionReturn;
use crate::context::pointer::Pointer;
use crate::context::traits::evmla_function::IEVMLAFunction;
use crate::optimizer::settings::size_level::SizeLevel;
use crate::optimizer::Optimizer;

///
/// The LLVM IR generator function.
///
#[derive(Debug)]
pub struct Function<'ctx> {
    /// The high-level source code name.
    name: String,
    /// The LLVM function declaration.
    declaration: FunctionDeclaration<'ctx>,
    /// The stack representation.
    stack: HashMap<String, Pointer<'ctx, AddressSpace>>,
    /// The return value entity.
    r#return: FunctionReturn<'ctx, AddressSpace>,

    /// The entry block. Each LLVM IR functions must have an entry block.
    entry_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The return/leave block. LLVM IR functions may have multiple returning blocks, but it is
    /// more reasonable to have a single returning block and other high-level language returns
    /// jumping to it. This way it is easier to implement some additional checks and clean-ups
    /// before the returning.
    return_block: inkwell::basic_block::BasicBlock<'ctx>,

    /// The EVM legacy assembly compiler data.
    evmla_data: Option<FunctionEVMLAData<'ctx>>,
}

impl<'ctx> Function<'ctx> {
    /// The stack hashmap default capacity.
    const STACK_HASHMAP_INITIAL_CAPACITY: usize = 64;

    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        name: String,
        declaration: FunctionDeclaration<'ctx>,
        r#return: FunctionReturn<'ctx, AddressSpace>,

        entry_block: inkwell::basic_block::BasicBlock<'ctx>,
        return_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> Self {
        Self {
            name,
            declaration,
            stack: HashMap::with_capacity(Self::STACK_HASHMAP_INITIAL_CAPACITY),
            r#return,

            entry_block,
            return_block,

            evmla_data: None,
        }
    }

    ///
    /// Returns the function name reference.
    ///
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    ///
    /// Checks whether the function is defined outside of the front-end.
    ///
    pub fn is_name_external(name: &str) -> bool {
        name.starts_with("llvm.")
            || (name.starts_with("__") && name != crate::r#const::ENTRY_FUNCTION_NAME)
    }

    ///
    /// Returns the LLVM function declaration.
    ///
    pub fn declaration(&self) -> FunctionDeclaration<'ctx> {
        self.declaration
    }

    ///
    /// Returns the N-th parameter of the function.
    ///
    pub fn get_nth_param(&self, index: usize) -> inkwell::values::BasicValueEnum<'ctx> {
        self.declaration()
            .value
            .get_nth_param(index as u32)
            .expect("Always exists")
    }

    ///
    /// Sets the default attributes.
    ///
    /// The attributes only affect the LLVM optimizations.
    ///
    pub fn set_default_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: FunctionDeclaration<'ctx>,
        optimizer: &Optimizer,
    ) {
        if optimizer.settings().level_middle_end == inkwell::OptimizationLevel::None {
            declaration.value.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                llvm.create_enum_attribute(Attribute::OptimizeNone as u32, 0),
            );
            declaration.value.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                llvm.create_enum_attribute(Attribute::NoInline as u32, 0),
            );
        }

        if optimizer.settings().level_middle_end_size == SizeLevel::Z {
            declaration.value.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                llvm.create_enum_attribute(Attribute::OptimizeForSize as u32, 0),
            );
            declaration.value.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                llvm.create_enum_attribute(Attribute::MinSize as u32, 0),
            );
        }

        declaration.value.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_string_attribute(
                StringAttribute::TargetFeatures.to_string().as_str(),
                format!("+{}", solx_utils::EVMVersion::Osaka).as_str(),
            ),
        );
        declaration.value.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_enum_attribute(Attribute::NoFree as u32, 0),
        );
        declaration.value.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_enum_attribute(Attribute::NullPointerIsValid as u32, 0),
        );
    }

    ///
    /// Sets the front-end runtime attributes.
    ///
    pub fn set_frontend_runtime_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: FunctionDeclaration<'ctx>,
        optimizer: &Optimizer,
    ) {
        if optimizer.settings().level_middle_end_size == SizeLevel::Z {
            declaration.value.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                llvm.create_enum_attribute(Attribute::NoInline as u32, 0),
            );
        }
    }

    ///
    /// Sets the exception handler attributes.
    ///
    pub fn set_exception_handler_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: FunctionDeclaration<'ctx>,
    ) {
        declaration.value.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_enum_attribute(Attribute::NoInline as u32, 0),
        );
    }

    ///
    /// Sets the size optimization attributes.
    ///
    pub fn set_size_attributes(
        llvm: &'ctx inkwell::context::Context,
        function: inkwell::values::FunctionValue<'ctx>,
    ) {
        function.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_enum_attribute(Attribute::OptimizeForSize as u32, 0),
        );
        function.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_enum_attribute(Attribute::MinSize as u32, 0),
        );
    }

    ///
    /// Saves the pointer to a stack variable, returning the pointer to the shadowed variable,
    /// if it exists.
    ///
    pub fn insert_stack_pointer(
        &mut self,
        name: String,
        pointer: Pointer<'ctx, AddressSpace>,
    ) -> Option<Pointer<'ctx, AddressSpace>> {
        self.stack.insert(name, pointer)
    }

    ///
    /// Gets the pointer to a stack variable.
    ///
    pub fn get_stack_pointer(&self, name: &str) -> Option<Pointer<'ctx, AddressSpace>> {
        self.stack.get(name).copied()
    }

    ///
    /// Removes the pointer to a stack variable.
    ///
    pub fn remove_stack_pointer(&mut self, name: &str) {
        self.stack.remove(name);
    }

    ///
    /// Returns the return entity representation.
    ///
    pub fn r#return(&self) -> FunctionReturn<'ctx, AddressSpace> {
        self.r#return
    }

    ///
    /// Returns the pointer to the function return value.
    ///
    /// # Panics
    /// If the pointer has not been set yet.
    ///
    pub fn return_pointer(&self) -> Option<Pointer<'ctx, AddressSpace>> {
        self.r#return.return_pointer()
    }

    ///
    /// Returns the return data size in bytes, based on the default stack alignment.
    ///
    /// # Panics
    /// If the pointer has not been set yet.
    ///
    pub fn return_data_size(&self) -> usize {
        self.r#return.return_data_size()
    }

    ///
    /// Returns the function entry block.
    ///
    pub fn entry_block(&self) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.entry_block
    }

    ///
    /// Returns the function return block.
    ///
    pub fn return_block(&self) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.return_block
    }

    ///
    /// Sets the EVM legacy assembly data.
    ///
    pub fn set_evmla_data(&mut self, data: FunctionEVMLAData<'ctx>) {
        self.evmla_data = Some(data);
    }

    ///
    /// Returns the EVM legacy assembly data reference.
    ///
    /// # Panics
    /// If the EVM data has not been initialized.
    ///
    pub fn evmla(&self) -> &FunctionEVMLAData<'ctx> {
        self.evmla_data
            .as_ref()
            .expect("The EVM data must have been initialized")
    }

    ///
    /// Returns the EVM legacy assembly data mutable reference.
    ///
    /// # Panics
    /// If the EVM data has not been initialized.
    ///
    pub fn evmla_mut(&mut self) -> &mut FunctionEVMLAData<'ctx> {
        self.evmla_data
            .as_mut()
            .expect("The EVM data must have been initialized")
    }
}

impl<'ctx> IEVMLAFunction<'ctx> for Function<'ctx> {
    fn find_block(&self, key: &BlockKey, stack_hash: &u64) -> anyhow::Result<Block<'ctx>> {
        let evmla_data = self.evmla();

        if evmla_data
            .blocks
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Undeclared function block {key}"))?
            .len()
            == 1
        {
            return evmla_data
                .blocks
                .get(key)
                .ok_or_else(|| anyhow::anyhow!("Undeclared function block {key}"))?
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Undeclared function block {key}"));
        }

        evmla_data
            .blocks
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Undeclared function block {key}"))?
            .iter()
            .find(|block| {
                block
                    .evm()
                    .stack_hashes
                    .iter()
                    .any(|hash| hash == stack_hash)
            })
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Undeclared function block {key}"))
    }
}
