//!
//! A Yul function definition: `yul.func` and the entry block its body is emitted into.
//!

use melior::ir::Block as MlirBlock;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;

use crate::Block;
use crate::Context;
use crate::Type;
use crate::ir::yul::block::YulBlock;
use crate::ods::yul::FuncOperation;

/// A `yul.func` definition. Every Yul signature is words in, words out, so a function is fully
/// described by its symbol and those two counts.
pub struct YulFunction {
    /// The symbol the call sites name, which is the function's Yul name.
    pub name: String,
    /// How many words the function takes.
    pub parameters: usize,
    /// How many words the function returns.
    pub results: usize,
}

impl YulFunction {
    /// Records a Yul function's symbol and word counts.
    pub fn new(name: String, parameters: usize, results: usize) -> Self {
        Self {
            name,
            parameters,
            results,
        }
    }

    /// Emits this function's `yul.func` into `body` - the `sol.inline_asm` region, which is the
    /// symbol table Yul functions live in - and returns the entry block its body is emitted into.
    /// The entry block's arguments carry the parameter words.
    pub fn define<'context>(
        &self,
        context: &Context<'context>,
        body: Block<'context>,
    ) -> YulBlock<'context> {
        let word = Type::yul_word(context.melior).into_mlir();
        let parameters = vec![word; self.parameters];
        let function_type =
            FunctionType::new(context.melior, &parameters, &vec![word; self.results]);
        let entry_arguments: Vec<_> = parameters
            .iter()
            .map(|parameter| (*parameter, context.location()))
            .collect();
        let region = Region::new();
        region.append_block(MlirBlock::new(&entry_arguments));

        let operation = body.append_operation(
            FuncOperation::builder(context.melior, context.location())
                .sym_name(StringAttribute::new(context.melior, self.name.as_str()))
                .function_type(TypeAttribute::new(function_type.into()))
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
