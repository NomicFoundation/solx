//!
//! The Ethereal IR block.
//!

pub mod element;

use std::collections::BTreeSet;

use crate::assembly::instruction::Instruction;
use crate::assembly::instruction::name::Name as InstructionName;

use self::element::Element;
use self::element::stack::Stack as ElementStack;

///
/// The Ethereal IR block.
///
#[derive(Debug, Clone)]
pub struct Block {
    /// The block key.
    pub key: solx_codegen_evm::BlockKey,
    /// The block instance.
    pub instance: Option<usize>,
    /// The block elements relevant to the stack consistency.
    pub elements: Vec<Element>,
    /// The block predecessors.
    pub predecessors: BTreeSet<(solx_codegen_evm::BlockKey, usize)>,
    /// The initial stack state.
    pub initial_stack: ElementStack,
    /// The stack.
    pub stack: ElementStack,
    /// The extra block hashes for alternative routes.
    pub extra_hashes: Vec<u64>,
}

impl Block {
    /// The elements vector initial capacity.
    pub const ELEMENTS_VECTOR_DEFAULT_CAPACITY: usize = 64;

    ///
    /// Assembles a block from the sequence of instructions.
    ///
    pub fn try_from_instructions(
        solc_version: semver::Version,
        code_segment: solx_utils::CodeSegment,
        slice: &[Instruction],
    ) -> anyhow::Result<(Self, usize)> {
        let mut cursor = 0;

        let tag: u64 = match slice[cursor].name {
            InstructionName::Tag => {
                let tag_value = slice[cursor]
                    .value
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Tag instruction missing value"))?;
                let tag = tag_value
                    .parse()
                    .map_err(|error| anyhow::anyhow!("Invalid Tag value `{tag_value}`: {error}"))?;
                cursor += 1;
                tag
            }
            _ => 0u64,
        };

        let mut block = Self {
            key: solx_codegen_evm::BlockKey::new(code_segment, tag),
            instance: None,
            elements: Vec::with_capacity(Self::ELEMENTS_VECTOR_DEFAULT_CAPACITY),
            predecessors: BTreeSet::new(),
            initial_stack: ElementStack::new(),
            stack: ElementStack::new(),
            extra_hashes: vec![],
        };

        let mut dead_code = false;
        while cursor < slice.len() {
            if !dead_code {
                let element: Element = Element::new(solc_version.clone(), slice[cursor].to_owned());
                block.elements.push(element);
            }

            match slice[cursor].name {
                InstructionName::RETURN
                | InstructionName::REVERT
                | InstructionName::STOP
                | InstructionName::INVALID => {
                    cursor += 1;
                    dead_code = true;
                }
                InstructionName::JUMP => {
                    cursor += 1;
                    dead_code = true;
                }
                InstructionName::Tag => {
                    break;
                }
                _ => {
                    cursor += 1;
                }
            }
        }

        Ok((block, cursor))
    }

    ///
    /// Inserts a predecessor tag.
    ///
    pub fn insert_predecessor(&mut self, key: solx_codegen_evm::BlockKey, instance: usize) {
        self.predecessors.insert((key, instance));
    }
}

impl solx_codegen_evm::WriteLLVM for Block {
    fn into_llvm(self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        for element in self.elements.into_iter() {
            element.into_llvm(context)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "block_{}/{}: {}",
            self.key,
            self.instance.unwrap_or_default(),
            if self.predecessors.is_empty() {
                "".to_owned()
            } else {
                format!(
                    "(predecessors: {})",
                    self.predecessors
                        .iter()
                        .map(|(key, instance)| format!("{key}/{instance}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            },
        )?;
        for element in self.elements.iter() {
            writeln!(f, "    {element}")?;
        }
        Ok(())
    }
}
