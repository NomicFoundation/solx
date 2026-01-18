//!
//! The LLVM pointer.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::context::traits::address_space::IAddressSpace;
use crate::context::IContext;

///
/// The LLVM pointer.
///
#[derive(Debug, Clone, Copy)]
pub struct Pointer<'ctx, AS>
where
    AS: IAddressSpace
        + Clone
        + Copy
        + PartialEq
        + Eq
        + Into<inkwell::AddressSpace>
        + std::fmt::Debug,
{
    /// The pointee type.
    pub r#type: inkwell::types::BasicTypeEnum<'ctx>,
    /// The address space.
    pub address_space: AS,
    /// The pointer value.
    pub value: inkwell::values::PointerValue<'ctx>,
}

impl<'ctx, AS> Pointer<'ctx, AS>
where
    AS: IAddressSpace
        + Clone
        + Copy
        + PartialEq
        + Eq
        + Into<inkwell::AddressSpace>
        + std::fmt::Debug,
{
    ///
    /// A shortcut constructor.
    ///
    pub fn new<T>(r#type: T, address_space: AS, value: inkwell::values::PointerValue<'ctx>) -> Self
    where
        T: BasicType<'ctx>,
    {
        Self {
            r#type: r#type.as_basic_type_enum(),
            address_space,
            value,
        }
    }

    ///
    /// Wraps a 256-bit primitive type pointer.
    ///
    pub fn new_stack_field<C>(context: &C, value: inkwell::values::PointerValue<'ctx>) -> Self
    where
        C: IContext<'ctx>,
    {
        Self {
            r#type: context.field_type().as_basic_type_enum(),
            address_space: AS::stack(),
            value,
        }
    }

    ///
    /// Creates a new pointer with the specified `offset`.
    ///
    pub fn new_with_offset<C, T>(
        context: &C,
        address_space: AS,
        r#type: T,
        offset: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) -> anyhow::Result<Self>
    where
        C: IContext<'ctx>,
        T: BasicType<'ctx>,
    {
        assert_ne!(
            address_space,
            AS::stack(),
            "Stack pointers cannot be addressed"
        );

        let value =
            context.build_int_to_ptr(offset, context.ptr_type(address_space.into()), name)?;
        Ok(Self::new(r#type, address_space, value))
    }

    ///
    /// Casts the pointer into another type.
    ///
    pub fn cast<T>(self, r#type: T) -> Self
    where
        T: BasicType<'ctx>,
    {
        Self {
            r#type: r#type.as_basic_type_enum(),
            address_space: self.address_space,
            value: self.value,
        }
    }

    ///
    /// Converts the pointer to a value enum.
    ///
    pub fn as_basic_value_enum(self) -> inkwell::values::BasicValueEnum<'ctx> {
        self.value.as_basic_value_enum()
    }
}
