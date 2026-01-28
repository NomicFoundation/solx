//!
//! The if-conditional statement.
//!

use solx_codegen_evm::IContext;
use solx_codegen_evm::ISolidityData;

use crate::declare_wrapper;
use crate::yul::parser::dialect::era::EraDialect;
use crate::yul::parser::wrapper::Wrap;

declare_wrapper!(
    solx_yul::yul::parser::statement::if_conditional::IfConditional<EraDialect>,
    IfConditional
);

impl solx_codegen_evm::WriteLLVM for IfConditional {
    fn into_llvm(self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        if let Some((solidity_data, solc_location)) =
            context.solidity_mut().zip(self.0.solc_location)
        {
            solidity_data.set_debug_info_solc_location(solc_location);
        }

        let condition = self
            .0
            .condition
            .wrap()
            .into_llvm(context)?
            .expect("Always exists")
            .to_llvm()
            .into_int_value();
        let condition = context.build_bit_cast_instruction(
            inkwell::builder::Builder::build_int_z_extend_or_bit_cast,
            condition,
            context.field_type(),
            "if_condition_extended",
        )?;
        let condition = context.build_int_compare(
            inkwell::IntPredicate::NE,
            condition,
            context.field_const(0),
            "if_condition_compared",
        )?;
        let main_block = context.append_basic_block("if_main");
        let join_block = context.append_basic_block("if_join");
        context.build_conditional_branch(condition, main_block, join_block)?;
        context.set_basic_block(main_block);
        self.0.block.wrap().into_llvm(context)?;
        context.build_unconditional_branch(join_block)?;
        context.set_basic_block(join_block);

        Ok(())
    }
}
