//!
//! The LLVM module context trait.
//!

pub mod attribute;
pub mod function;
pub mod r#loop;
pub mod pointer;
pub mod traits;
pub mod value;

use std::cell::RefCell;
use std::rc::Rc;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::debug_config::DebugConfig;
use crate::optimizer::Optimizer;

use self::function::declaration::Declaration as FunctionDeclaration;
use self::r#loop::Loop;
use self::pointer::Pointer;
use self::traits::address_space::IAddressSpace;
use self::traits::evmla_data::IEVMLAData;
use self::traits::evmla_function::IEVMLAFunction;
use self::traits::solidity_data::ISolidityData;
use self::traits::yul_data::IYulData;

///
/// The LLVM module context trait.
///
pub trait IContext<'ctx> {
    ///
    /// The address space unique to each target.
    ///
    type AddressSpace: IAddressSpace
        + Clone
        + Copy
        + PartialEq
        + Eq
        + Into<inkwell::AddressSpace>
        + std::fmt::Debug;

    ///
    /// The function type.
    ///
    type Function: IEVMLAFunction<'ctx>;

    ///
    /// The Solidity extra data type.
    ///
    type SolidityData: ISolidityData;

    ///
    /// The Yul extra data type.
    ///
    type YulData: IYulData;

    ///
    /// The EVMLA extra data type.
    ///
    type EVMLAData: IEVMLAData<'ctx>;

    ///
    /// Returns the inner LLVM context.
    ///
    fn llvm(&self) -> &'ctx inkwell::context::Context;

    ///
    /// Returns the LLVM IR builder.
    ///
    fn builder(&self) -> &inkwell::builder::Builder<'ctx>;

    ///
    /// Returns the current LLVM IR module reference.
    ///
    fn module(&self) -> &inkwell::module::Module<'ctx>;

    ///
    /// Returns the optimizer reference.
    ///
    fn optimizer(&self) -> &Optimizer;

    ///
    /// Returns the debug config reference.
    ///
    fn debug_config(&self) -> Option<&DebugConfig>;

    ///
    /// Sets the code type.
    ///
    fn set_code_segment(&mut self, code_segment: solx_utils::CodeSegment);

    ///
    /// Returns the code type.
    ///
    fn code_segment(&self) -> Option<solx_utils::CodeSegment>;

    ///
    /// Returns the EVM version.
    ///
    fn evm_version(&self) -> solx_utils::EVMVersion;

    ///
    /// Appends a new basic block to the current function.
    ///
    fn append_basic_block(&self, name: &str) -> inkwell::basic_block::BasicBlock<'ctx>;

    ///
    /// Sets the current basic block.
    ///
    fn set_basic_block(&self, block: inkwell::basic_block::BasicBlock<'ctx>);

    ///
    /// Returns the current basic block.
    ///
    fn basic_block(&self) -> inkwell::basic_block::BasicBlock<'ctx>;

    ///
    /// Checks if the current basic block is terminated.
    ///
    fn is_basic_block_terminated(&self) -> bool;

    ///
    /// Pushes a new loop context to the stack.
    ///
    fn push_loop(
        &mut self,
        body_block: inkwell::basic_block::BasicBlock<'ctx>,
        continue_block: inkwell::basic_block::BasicBlock<'ctx>,
        join_block: inkwell::basic_block::BasicBlock<'ctx>,
    );

    ///
    /// Pops the current loop context from the stack.
    ///
    fn pop_loop(&mut self);

    ///
    /// Returns the current loop context.
    ///
    fn r#loop(&self) -> &Loop<'ctx>;

    ///
    /// Appends a function to the current module.
    ///
    fn add_function(
        &mut self,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
        return_values_length: usize,
        linkage: Option<inkwell::module::Linkage>,
    ) -> anyhow::Result<Rc<RefCell<Self::Function>>>;

    ///
    /// Returns a shared reference to the specified function.
    ///
    fn get_function(&self, name: &str) -> Option<Rc<RefCell<Self::Function>>>;

    ///
    /// Returns a shared reference to the current active function.
    ///
    fn current_function(&self) -> Rc<RefCell<Self::Function>>;

    ///
    /// Sets the current active function.
    ///
    fn set_current_function(&mut self, name: &str) -> anyhow::Result<()>;

    ///
    /// Builds a stack allocation instruction.
    ///
    /// Sets the alignment to 256 bits.
    ///
    fn build_alloca<T>(
        &self,
        r#type: T,
        name: &str,
    ) -> anyhow::Result<Pointer<'ctx, Self::AddressSpace>>
    where
        T: BasicType<'ctx> + Clone + Copy,
    {
        let pointer = self.builder().build_alloca(r#type, name)?;
        self.basic_block()
            .get_last_instruction()
            .expect("Always exists")
            .set_alignment(solx_utils::BYTE_LENGTH_FIELD as u32)
            .map_err(|error| anyhow::anyhow!(error))?;
        Ok(Pointer::new(r#type, Self::AddressSpace::stack(), pointer))
    }

    ///
    /// Builds a stack load instruction.
    ///
    /// Sets the alignment to 256 bits for the stack and 1 bit for the heap, parent, and child.
    ///
    fn build_load(
        &self,
        pointer: Pointer<'ctx, Self::AddressSpace>,
        name: &str,
    ) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
        let value = self
            .builder()
            .build_load(pointer.r#type, pointer.value, name)?;

        let alignment = if Self::AddressSpace::stack() == pointer.address_space {
            solx_utils::BYTE_LENGTH_FIELD
        } else {
            solx_utils::BYTE_LENGTH_BYTE
        };

        self.basic_block()
            .get_last_instruction()
            .expect("Always exists")
            .set_alignment(alignment as u32)
            .map_err(|error| anyhow::anyhow!(error))?;
        Ok(value)
    }

    ///
    /// Builds a stack store instruction.
    ///
    /// Sets the alignment to 256 bits for the stack and 1 bit for the heap, parent, and child.
    ///
    fn build_store<V>(
        &self,
        pointer: Pointer<'ctx, Self::AddressSpace>,
        value: V,
    ) -> anyhow::Result<()>
    where
        V: BasicValue<'ctx>,
    {
        let instruction = self.builder().build_store(pointer.value, value)?;

        let alignment = if Self::AddressSpace::stack() == pointer.address_space {
            solx_utils::BYTE_LENGTH_FIELD
        } else {
            solx_utils::BYTE_LENGTH_BYTE
        };

        instruction
            .set_alignment(alignment as u32)
            .map_err(|error| anyhow::anyhow!(error))?;
        Ok(())
    }

    ///
    /// Builds a GEP instruction.
    ///
    fn build_gep<T>(
        &self,
        pointer: Pointer<'ctx, Self::AddressSpace>,
        indexes: &[inkwell::values::IntValue<'ctx>],
        element_type: T,
        name: &str,
    ) -> anyhow::Result<Pointer<'ctx, Self::AddressSpace>>
    where
        T: BasicType<'ctx>,
    {
        let value = unsafe {
            self.builder()
                .build_gep(pointer.r#type, pointer.value, indexes, name)?
        };
        Ok(Pointer::new(element_type, pointer.address_space, value))
    }

    ///
    /// Builds a conditional branch.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    fn build_conditional_branch(
        &self,
        comparison: inkwell::values::IntValue<'ctx>,
        then_block: inkwell::basic_block::BasicBlock<'ctx>,
        else_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> anyhow::Result<()> {
        if self.basic_block().get_terminator().is_some() {
            return Ok(());
        }

        self.builder()
            .build_conditional_branch(comparison, then_block, else_block)?;
        Ok(())
    }

    ///
    /// Builds an unconditional branch.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    fn build_unconditional_branch(
        &self,
        destination_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> anyhow::Result<()> {
        if self.basic_block().get_terminator().is_some() {
            return Ok(());
        }

        self.builder()
            .build_unconditional_branch(destination_block)?;
        Ok(())
    }

    ///
    /// Builds a call.
    ///
    fn build_call(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicValueEnum<'ctx>],
        name: &str,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>;

    ///
    /// Builds a call with metadata arguments.
    ///
    fn build_call_metadata(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>;

    ///
    /// Builds an invoke.
    ///
    /// Is defaulted to a call if there is no global exception handler.
    ///
    fn build_invoke(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicValueEnum<'ctx>],
        name: &str,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>;

    ///
    /// Builds a memory copy call.
    ///
    /// Sets the alignment to `1`, since all non-stack memory pages have such alignment.
    ///
    fn build_memcpy(
        &self,
        function: FunctionDeclaration<'ctx>,
        destination: Pointer<'ctx, Self::AddressSpace>,
        source: Pointer<'ctx, Self::AddressSpace>,
        size: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) -> anyhow::Result<()> {
        let call_site_value = self.builder().build_indirect_call(
            function.r#type,
            function.value.as_global_value().as_pointer_value(),
            &[
                destination.value.as_basic_value_enum().into(),
                source.value.as_basic_value_enum().into(),
                size.as_basic_value_enum().into(),
                self.bool_type().const_zero().as_basic_value_enum().into(),
            ],
            name,
        )?;

        call_site_value.set_alignment_attribute(inkwell::attributes::AttributeLoc::Param(0), 1);
        call_site_value.set_alignment_attribute(inkwell::attributes::AttributeLoc::Param(1), 1);
        Ok(())
    }

    ///
    /// Builds a return.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    fn build_return(&self, value: Option<&dyn BasicValue<'ctx>>) -> anyhow::Result<()> {
        if self.basic_block().get_terminator().is_some() {
            return Ok(());
        }

        self.builder().build_return(value)?;
        Ok(())
    }

    ///
    /// Builds an unreachable.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    fn build_unreachable(&self) -> anyhow::Result<()> {
        if self.basic_block().get_terminator().is_some() {
            return Ok(());
        }

        self.builder().build_unreachable()?;
        Ok(())
    }

    ///
    /// Returns a boolean type constant.
    ///
    fn bool_const(&self, value: bool) -> inkwell::values::IntValue<'ctx> {
        self.bool_type().const_int(u64::from(value), false)
    }

    ///
    /// Returns an integer type constant.
    ///
    fn integer_const(&self, bit_length: usize, value: u64) -> inkwell::values::IntValue<'ctx> {
        self.integer_type(bit_length).const_int(value, false)
    }

    ///
    /// Returns a 256-bit field type constant.
    ///
    fn field_const(&self, value: u64) -> inkwell::values::IntValue<'ctx> {
        self.field_type().const_int(value, false)
    }

    ///
    /// Returns a 256-bit field type undefined value.
    ///
    fn field_undef(&self) -> inkwell::values::IntValue<'ctx> {
        self.field_type().get_undef()
    }

    ///
    /// Returns a field type constant from a decimal string.
    ///
    fn field_const_str_dec(&self, value: &str) -> inkwell::values::IntValue<'ctx> {
        self.field_type()
            .const_int_from_string(value, inkwell::types::StringRadix::Decimal)
            .unwrap_or_else(|| panic!("Invalid string constant `{value}`"))
    }

    ///
    /// Returns a field type constant from a hexadecimal string.
    ///
    fn field_const_str_hex(&self, value: &str) -> inkwell::values::IntValue<'ctx> {
        self.field_type()
            .const_int_from_string(
                value.strip_prefix("0x").unwrap_or(value),
                inkwell::types::StringRadix::Hexadecimal,
            )
            .unwrap_or_else(|| panic!("Invalid string constant `{value}`"))
    }

    ///
    /// Returns the void type.
    ///
    fn void_type(&self) -> inkwell::types::VoidType<'ctx> {
        self.llvm().void_type()
    }

    ///
    /// Returns the boolean type.
    ///
    fn bool_type(&self) -> inkwell::types::IntType<'ctx> {
        self.llvm().bool_type()
    }

    ///
    /// Returns the default byte type.
    ///
    fn byte_type(&self) -> inkwell::types::IntType<'ctx> {
        self.integer_type(solx_utils::BIT_LENGTH_BYTE)
    }

    ///
    /// Returns the integer type of the specified bit-length.
    ///
    fn integer_type(&self, bit_length: usize) -> inkwell::types::IntType<'ctx> {
        self.llvm().custom_width_int_type(bit_length as u32)
    }

    ///
    /// Returns the default field type.
    ///
    fn field_type(&self) -> inkwell::types::IntType<'ctx> {
        self.integer_type(solx_utils::BIT_LENGTH_FIELD)
    }

    ///
    /// Returns the pointer type with a specified address space.
    ///
    fn ptr_type(&self, address_space: inkwell::AddressSpace) -> inkwell::types::PointerType<'ctx> {
        self.llvm().ptr_type(address_space)
    }

    ///
    /// Returns the array type with the specified length.
    ///
    fn array_type<T>(&self, element_type: T, length: usize) -> inkwell::types::ArrayType<'ctx>
    where
        T: BasicType<'ctx>,
    {
        element_type.array_type(length as u32)
    }

    ///
    /// Returns the structure type with specified fields.
    ///
    fn structure_type<T>(&self, field_types: &[T]) -> inkwell::types::StructType<'ctx>
    where
        T: BasicType<'ctx>,
    {
        let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> =
            field_types.iter().map(T::as_basic_type_enum).collect();
        self.llvm().struct_type(field_types.as_slice(), false)
    }

    ///
    /// Sets the Solidity data.
    ///
    fn set_solidity_data(&mut self, data: Self::SolidityData);

    ///
    /// Returns the Solidity data reference.
    ///
    /// # Panics
    /// If the Solidity data has not been initialized.
    ///
    fn solidity(&self) -> Option<&Self::SolidityData>;

    ///
    /// Returns the Solidity data mutable reference.
    ///
    /// # Panics
    /// If the Solidity data has not been initialized.
    ///
    fn solidity_mut(&mut self) -> Option<&mut Self::SolidityData>;

    ///
    /// Sets the Yul data.
    ///
    fn set_yul_data(&mut self, data: Self::YulData);

    ///
    /// Returns the Yul data reference.
    ///
    /// # Panics
    /// If the Yul data has not been initialized.
    ///
    fn yul(&self) -> Option<&Self::YulData>;

    ///
    /// Returns the Yul data mutable reference.
    ///
    /// # Panics
    /// If the Yul data has not been initialized.
    ///
    fn yul_mut(&mut self) -> Option<&mut Self::YulData>;

    ///
    /// Sets the EVM legacy assembly data.
    ///
    fn set_evmla_data(&mut self, data: Self::EVMLAData);

    ///
    /// Returns the EVM legacy assembly data reference.
    ///
    /// # Panics
    /// If the EVM data has not been initialized.
    ///
    fn evmla(&self) -> Option<&Self::EVMLAData>;

    ///
    /// Returns the EVM legacy assembly data mutable reference.
    ///
    /// # Panics
    /// If the EVM data has not been initialized.
    ///
    fn evmla_mut(&mut self) -> Option<&mut Self::EVMLAData>;
}
