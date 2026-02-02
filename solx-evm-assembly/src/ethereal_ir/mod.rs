//!
//! The Ethereal IR of the EVM bytecode.
//!

pub mod function;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use solx_codegen_evm::IContext;

use crate::assembly::instruction::Instruction;
use crate::extra_metadata::ExtraMetadata;

use self::function::Function;
use self::function::block::Block;
use self::function::r#type::Type as FunctionType;

///
/// Ethereal IR of EVM bytecode.
///
/// Ethereal IR (EthIR) is a special IR between the EVM legacy assembly and LLVM IR. It is
/// created to facilitate the translation and provide an additional environment for applying
/// transformations, duplication parts of the call and control flow graphs, tracking the
/// data flow, and a few more algorithms of static analysis.
///
/// The most important feature of EthIR is flattening the block tags and duplicating blocks for
/// each of initial states of the stack. LLVM IR supports only static control flow, so the
/// stack state must be known all the way throughout the program.
///
#[derive(Debug)]
pub struct EtherealIR {
    /// Entry function.
    pub entry_function: Function,
    /// Defined functions.
    pub defined_functions: BTreeMap<solx_codegen_evm::BlockKey, Function>,
}

impl EtherealIR {
    /// The blocks hashmap initial capacity.
    pub const BLOCKS_HASHMAP_DEFAULT_CAPACITY: usize = 64;

    ///
    /// Assembles a sequence of functions from the sequence of instructions.
    ///
    pub fn new(
        solc_version: semver::Version,
        extra_metadata: ExtraMetadata,
        code_segment: Option<solx_utils::CodeSegment>,
        blocks: HashMap<solx_codegen_evm::BlockKey, Block>,
    ) -> anyhow::Result<Self> {
        let mut entry_function =
            Function::new(solc_version, code_segment, FunctionType::new_entry());
        let mut defined_functions = BTreeMap::new();
        let mut visited_functions = BTreeSet::new();
        entry_function.traverse(
            &blocks,
            &mut defined_functions,
            &extra_metadata,
            &mut visited_functions,
        )?;

        Ok(Self {
            entry_function,
            defined_functions,
        })
    }

    ///
    /// Gets blocks for the specified type of the contract code.
    ///
    pub fn get_blocks(
        solc_version: semver::Version,
        code_segment: solx_utils::CodeSegment,
        instructions: &[Instruction],
    ) -> anyhow::Result<HashMap<solx_codegen_evm::BlockKey, Block>> {
        let mut blocks = HashMap::with_capacity(Self::BLOCKS_HASHMAP_DEFAULT_CAPACITY);
        let mut offset = 0;

        while offset < instructions.len() {
            let (block, size) = Block::try_from_instructions(
                solc_version.clone(),
                code_segment,
                &instructions[offset..],
            )?;
            blocks.insert(
                solx_codegen_evm::BlockKey::new(code_segment, block.key.tag.clone()),
                block,
            );
            offset += size;
        }

        Ok(blocks)
    }
}

impl solx_codegen_evm::WriteLLVM for EtherealIR {
    fn declare(&mut self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        self.entry_function.declare(context)?;

        for (_key, function) in self.defined_functions.iter_mut() {
            function.declare(context)?;
        }

        Ok(())
    }

    fn into_llvm(self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        context.evmla_mut().expect("Always exists").stack = vec![];

        self.entry_function.into_llvm(context)?;

        for (_key, function) in self.defined_functions.into_iter() {
            function.into_llvm(context)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for EtherealIR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.entry_function)?;

        for (_key, function) in self.defined_functions.iter() {
            writeln!(f, "{function}")?;
        }

        Ok(())
    }
}
