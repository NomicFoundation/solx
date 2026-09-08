//!
//! Yul function call resolution metadata.
//!

use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;

use crate::Block;
use crate::Context;
use crate::FunctionType;
use crate::Type;
use crate::YulBlock;
use crate::ods::yul::FuncOperation;

/// A Yul function a call site can name: its symbol and MLIR-interned signature. Every Yul
/// signature is words in, words out.
#[derive(Clone)]
pub struct YulFunction<'context> {
    /// The symbol the call sites name.
    pub mlir_name: String,
    /// Parameter and result types, MLIR-interned.
    pub function_type: FunctionType<'context>,
}

impl<'context> YulFunction<'context> {
    /// Records a Yul function's symbol and the signature its parameter and return counts spell out.
    pub fn new(
        mlir_name: String,
        parameters: usize,
        results: usize,
        context: &Context<'context>,
    ) -> Self {
        let word = Type::yul_word(context.melior);
        Self {
            mlir_name,
            function_type: FunctionType {
                parameters: vec![word; parameters],
                results: vec![word; results],
            },
        }
    }

    /// Emits this function's `yul.func` at `position` in `assembly_body` - the `sol.inline_asm`
    /// region, which is the symbol table Yul functions live in - and returns the entry block its
    /// body is emitted into. The entry block's arguments carry the parameter words.
    pub fn define(
        &self,
        position: usize,
        context: &Context<'context>,
        assembly_body: Block<'context>,
    ) -> YulBlock<'context> {
        let (body, entry) = Block::region(&self.function_type.parameters, context);
        assembly_body.insert_operation(
            position,
            FuncOperation::builder(context.melior, context.location())
                .sym_name(StringAttribute::new(
                    context.melior,
                    self.mlir_name.as_str(),
                ))
                .function_type(TypeAttribute::new(
                    self.function_type.to_mlir(context.melior).into(),
                ))
                .body(body)
                .build()
                .into(),
        );
        YulBlock::from(entry)
    }
}
