//!
//! Yul function call resolution metadata.
//!

use melior::ir::Block as MlirBlock;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;

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

    /// Emits this function's `yul.func` at `position` in `body` - the `sol.inline_asm` region, which
    /// is the symbol table Yul functions live in - and returns the entry block its body is emitted
    /// into. The entry block's arguments carry the parameter words.
    pub fn define(
        &self,
        position: usize,
        context: &Context<'context>,
        body: Block<'context>,
    ) -> YulBlock<'context> {
        let entry_arguments: Vec<_> = self
            .function_type
            .parameters
            .iter()
            .map(|parameter| (parameter.into_mlir(), context.location()))
            .collect();
        let region = Region::new();
        region.append_block(MlirBlock::new(&entry_arguments));

        let operation = body.insert_operation(
            position,
            FuncOperation::builder(context.melior, context.location())
                .sym_name(StringAttribute::new(
                    context.melior,
                    self.mlir_name.as_str(),
                ))
                .function_type(TypeAttribute::new(
                    self.function_type.to_mlir(context.melior).into(),
                ))
                .body(region)
                .build()
                .into(),
        );
        YulBlock::from(
            operation
                .region(0)
                .expect("yul.func has one region")
                .first_block()
                .expect("yul.func body has an entry block"),
        )
    }
}
