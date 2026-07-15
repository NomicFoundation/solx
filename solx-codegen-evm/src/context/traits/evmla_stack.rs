//!
//! The LLVM IR EVMLA stack access trait.
//!

use crate::context::IContext;
use crate::context::pointer::Pointer;
use crate::context::traits::evmla_data::IEVMLAData;
use crate::context::traits::evmla_data::shadow_slot::ShadowSlot;

///
/// The LLVM IR EVMLA stack access trait.
///
/// Resolves stack positions through the block-local shadow stack, emitting frame loads and stores
/// only where the shadow cannot answer from an SSA value.
///
pub trait IEVMLAStack<'ctx>: IContext<'ctx> + Sized {
    ///
    /// Reads the value at the given stack position, materializing it from the function frame if it
    /// has not yet been produced as an SSA value within the current block.
    ///
    fn evmla_stack_read(
        &self,
        position: usize,
    ) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
        match self.evmla().expect("Always exists").shadow_peek(position) {
            ShadowSlot::Value(value) => Ok(value),
            ShadowSlot::Memory(index) => {
                let pointer = self
                    .evmla()
                    .expect("Always exists")
                    .get_element(index)
                    .to_llvm()
                    .into_pointer_value();
                self.build_load(Pointer::new_stack_field(self, pointer), "stack_value")
            }
        }
    }

    ///
    /// Reconciles the block-local shadow stack into the function frame for the live `depth` at a
    /// block exit, so that successor blocks read their input stack through memory. Positions still
    /// resolving to their own slot are skipped; the first pass loads every relocated slot before
    /// any store happens, so permutations are materialized correctly.
    ///
    fn evmla_stack_flush(&mut self, depth: usize) -> anyhow::Result<()> {
        for position in 0..depth {
            if let ShadowSlot::Memory(index) =
                self.evmla().expect("Always exists").shadow_peek(position)
                && index != position
            {
                let value = self.evmla_stack_read(position)?;
                self.evmla_mut()
                    .expect("Always exists")
                    .shadow_write(position, value);
            }
        }
        for position in 0..depth {
            if let ShadowSlot::Value(value) =
                self.evmla().expect("Always exists").shadow_peek(position)
            {
                let pointer = self
                    .evmla()
                    .expect("Always exists")
                    .get_element(position)
                    .to_llvm()
                    .into_pointer_value();
                self.build_store(Pointer::new_stack_field(self, pointer), value)?;
            }
        }
        Ok(())
    }
}

impl<'ctx, C> IEVMLAStack<'ctx> for C where C: IContext<'ctx> {}
