//!
//! The Ethereal IR function.
//!

pub mod block;
pub mod queue_element;
pub mod r#type;
pub mod visited_element;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;
use num::CheckedAdd;
use num::CheckedDiv;
use num::CheckedMul;
use num::CheckedSub;
use num::Num;
use num::One;
use num::ToPrimitive;
use num::Zero;

use solx_codegen_evm::IContext;
use solx_codegen_evm::IEVMLAFunction;

use crate::assembly::instruction::Instruction;
use crate::assembly::instruction::name::Name as InstructionName;
use crate::ethereal_ir::function::block::element::stack::Stack;
use crate::ethereal_ir::function::block::element::stack::element::Element;
use crate::extra_metadata::ExtraMetadata;
use crate::extra_metadata::defined_function::DefinedFunction as ExtraMetadataDefinedFunction;

use self::block::Block;
use self::block::element::Element as BlockElement;
use self::block::element::stack::element::Element as StackElement;
use self::queue_element::QueueElement;
use self::r#type::Type;
use self::visited_element::VisitedElement;

///
/// The Ethereal IR function.
///
#[derive(Debug)]
pub struct Function {
    /// The Solidity compiler version.
    pub solc_version: semver::Version,
    /// The function name.
    pub name: String,
    /// The optional code segment. Only used for the EVM target.
    pub code_segment: Option<solx_utils::CodeSegment>,
    /// The separately labelled blocks.
    pub blocks: BTreeMap<solx_codegen_evm::BlockKey, Vec<Block>>,
    /// Index of initial stack hashes per block key for O(1) duplicate detection.
    block_hash_index: HashMap<solx_codegen_evm::BlockKey, HashSet<u64>>,
    /// The function type.
    pub r#type: Type,
    /// The function stack size.
    pub stack_size: usize,
}

impl Function {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        solc_version: semver::Version,
        code_segment: Option<solx_utils::CodeSegment>,
        r#type: Type,
    ) -> Self {
        let name = match r#type {
            Type::Entry => solx_codegen_evm::ENTRY_FUNCTION_NAME.to_string(),
            Type::Defined {
                ref name,
                ref block_key,
                ..
            } => format!("{name}_{block_key}"),
        };

        Self {
            solc_version,
            name,
            code_segment,
            blocks: BTreeMap::new(),
            block_hash_index: HashMap::new(),
            r#type,
            stack_size: 0,
        }
    }

    ///
    /// Runs the function block traversal.
    ///
    pub fn traverse(
        &mut self,
        blocks: &HashMap<solx_codegen_evm::BlockKey, Block>,
        functions: &mut BTreeMap<solx_codegen_evm::BlockKey, Self>,
        extra_metadata: &ExtraMetadata,
        visited_functions: &mut BTreeSet<VisitedElement>,
    ) -> anyhow::Result<()> {
        let mut visited_blocks = BTreeSet::new();

        let code_segments = match self.code_segment {
            Some(code_segment) => vec![code_segment],
            None => vec![
                solx_utils::CodeSegment::Deploy,
                solx_utils::CodeSegment::Runtime,
            ],
        };

        match self.r#type {
            Type::Entry => {
                for code_segment in code_segments {
                    self.consume_block(
                        blocks,
                        functions,
                        extra_metadata,
                        visited_functions,
                        &mut visited_blocks,
                        QueueElement::new(
                            solx_codegen_evm::BlockKey::new(code_segment, num::BigUint::zero()),
                            None,
                            Stack::new(),
                        ),
                    )?;
                }
            }
            Type::Defined {
                ref block_key,
                input_size,
                output_size,
                ..
            } => {
                let mut stack = Stack::with_capacity(1 + input_size);
                stack.push(Element::ReturnAddress(output_size));
                stack.append(&mut Stack::new_with_elements(vec![
                    Element::value(
                        "ARGUMENT".to_owned()
                    );
                    input_size
                ]));

                self.consume_block(
                    blocks,
                    functions,
                    extra_metadata,
                    visited_functions,
                    &mut visited_blocks,
                    QueueElement::new(block_key.to_owned(), None, stack),
                )?;
            }
        }

        self.finalize();

        Ok(())
    }

    ///
    /// Consumes the entry or a conditional block attached to another one.
    ///
    fn consume_block(
        &mut self,
        blocks: &HashMap<solx_codegen_evm::BlockKey, Block>,
        functions: &mut BTreeMap<solx_codegen_evm::BlockKey, Self>,
        extra_metadata: &ExtraMetadata,
        visited_functions: &mut BTreeSet<VisitedElement>,
        visited_blocks: &mut BTreeSet<VisitedElement>,
        mut queue_element: QueueElement,
    ) -> anyhow::Result<()> {
        let version = self.solc_version.to_owned();

        let mut queue = vec![];

        let mut block = blocks
            .get(&queue_element.block_key)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!("Undeclared destination block {}", queue_element.block_key)
            })?;
        block.initial_stack = queue_element.stack.clone();
        let block = self.insert_block(block);
        block.stack = block.initial_stack.clone();
        if let Some(predecessor) = queue_element.predecessor.take() {
            block.insert_predecessor(predecessor.0, predecessor.1);
        }

        let visited_element =
            VisitedElement::new(queue_element.block_key.clone(), queue_element.stack.hash());
        if visited_blocks.contains(&visited_element) {
            return Ok(());
        }
        visited_blocks.insert(visited_element);

        let mut block_size = 0;
        for block_element in block.elements.iter_mut() {
            block_size += 1;

            if Self::handle_instruction(
                blocks,
                functions,
                extra_metadata,
                visited_functions,
                block.key.code_segment,
                block.instance.unwrap_or_default(),
                &mut block.stack,
                block_element,
                &version,
                &mut queue,
                &mut queue_element,
            )
            .is_err()
            {
                block_element.instruction = Instruction::invalid(&block_element.instruction);
                block_element.stack = block.stack.clone();
                break;
            }
        }
        block.elements.truncate(block_size);

        for element in queue.into_iter() {
            self.consume_block(
                blocks,
                functions,
                extra_metadata,
                visited_functions,
                visited_blocks,
                element,
            )?;
        }

        Ok(())
    }

    ///
    /// Processes an instruction, returning an error, if there is an invalid stack state.
    ///
    /// The blocks with an invalid stack state are considered being partially unreachable, and
    /// the invalid part is truncated after terminating with an `INVALID` instruction.
    ///
    fn handle_instruction(
        blocks: &HashMap<solx_codegen_evm::BlockKey, Block>,
        functions: &mut BTreeMap<solx_codegen_evm::BlockKey, Self>,
        extra_metadata: &ExtraMetadata,
        visited_functions: &mut BTreeSet<VisitedElement>,
        code_segment: solx_utils::CodeSegment,
        instance: usize,
        block_stack: &mut Stack,
        block_element: &mut BlockElement,
        version: &semver::Version,
        queue: &mut Vec<QueueElement>,
        queue_element: &mut QueueElement,
    ) -> anyhow::Result<()> {
        let (stack_output, queue_element) = match &block_element.instruction {
            Instruction {
                name: InstructionName::PUSH_Tag,
                value: Some(tag),
                ..
            } => {
                let tag: num::BigUint = tag
                    .parse()
                    .map_err(|error| anyhow::anyhow!("Invalid PUSH_Tag value `{tag}`: {error}"))?;
                (vec![Element::Tag(tag & num::BigUint::from(u64::MAX))], None)
            }
            instruction @ Instruction {
                name: InstructionName::JUMP,
                ..
            } => {
                queue_element.predecessor = Some((queue_element.block_key.clone(), instance));

                let block_key = match block_stack
                    .elements
                    .last()
                    .ok_or_else(|| anyhow::anyhow!("Destination tag is missing"))?
                {
                    Element::Tag(destination) if destination > &num::BigUint::from(u32::MAX) => {
                        solx_codegen_evm::BlockKey::new(
                            solx_utils::CodeSegment::Runtime,
                            destination.to_owned() - num::BigUint::from(1u64 << 32),
                        )
                    }
                    Element::Tag(destination) => {
                        solx_codegen_evm::BlockKey::new(code_segment, destination.to_owned())
                    }
                    Element::ReturnAddress(output_size) => {
                        block_element.instruction =
                            Instruction::r#return(1 + output_size, instruction);
                        Self::update_io_data(block_stack, block_element, 1 + output_size, vec![])?;
                        return Ok(());
                    }
                    element => {
                        return Err(anyhow::anyhow!(
                            "The {} instruction expected a tag or return address, found {}",
                            instruction.name,
                            element
                        ));
                    }
                };

                let (next_block_key, stack_output) = if let Some(function) =
                    extra_metadata.get(block_key.code_segment, &block_key.tag)
                {
                    Self::handle_function_call(
                        function,
                        blocks,
                        functions,
                        extra_metadata,
                        visited_functions,
                        block_key,
                        block_stack,
                        block_element,
                        version,
                    )?
                } else {
                    (block_key, vec![])
                };

                (
                    stack_output,
                    Some(QueueElement::new(
                        next_block_key,
                        queue_element.predecessor.clone(),
                        Stack::new(),
                    )),
                )
            }
            instruction @ Instruction {
                name: InstructionName::JUMPI,
                ..
            } => {
                queue_element.predecessor = Some((queue_element.block_key.clone(), instance));

                let block_key = match block_stack
                    .elements
                    .last()
                    .ok_or_else(|| anyhow::anyhow!("Destination tag is missing"))?
                {
                    Element::Tag(destination) if destination > &num::BigUint::from(u32::MAX) => {
                        solx_codegen_evm::BlockKey::new(
                            solx_utils::CodeSegment::Runtime,
                            destination.to_owned() - num::BigUint::from(1u64 << 32),
                        )
                    }
                    Element::Tag(destination) => {
                        solx_codegen_evm::BlockKey::new(code_segment, destination.to_owned())
                    }
                    element => {
                        return Err(anyhow::anyhow!(
                            "The {} instruction expected a tag or return address, found {}",
                            instruction.name,
                            element
                        ));
                    }
                };

                (
                    vec![],
                    Some(QueueElement::new(
                        block_key,
                        queue_element.predecessor.clone(),
                        Stack::new(),
                    )),
                )
            }
            Instruction {
                name: InstructionName::Tag,
                value: Some(tag),
                ..
            } => {
                let tag: num::BigUint = tag
                    .parse()
                    .map_err(|error| anyhow::anyhow!("Invalid Tag value `{tag}`: {error}"))?;
                let block_key = solx_codegen_evm::BlockKey::new(code_segment, tag);

                queue_element.predecessor = Some((queue_element.block_key.clone(), instance));
                queue_element.block_key = block_key.clone();

                (
                    vec![],
                    Some(QueueElement::new(
                        block_key,
                        queue_element.predecessor.clone(),
                        Stack::new(),
                    )),
                )
            }

            Instruction {
                name: InstructionName::SWAP1,
                ..
            } => {
                block_stack.swap(1)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP2,
                ..
            } => {
                block_stack.swap(2)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP3,
                ..
            } => {
                block_stack.swap(3)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP4,
                ..
            } => {
                block_stack.swap(4)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP5,
                ..
            } => {
                block_stack.swap(5)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP6,
                ..
            } => {
                block_stack.swap(6)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP7,
                ..
            } => {
                block_stack.swap(7)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP8,
                ..
            } => {
                block_stack.swap(8)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP9,
                ..
            } => {
                block_stack.swap(9)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP10,
                ..
            } => {
                block_stack.swap(10)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP11,
                ..
            } => {
                block_stack.swap(11)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP12,
                ..
            } => {
                block_stack.swap(12)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP13,
                ..
            } => {
                block_stack.swap(13)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP14,
                ..
            } => {
                block_stack.swap(14)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP15,
                ..
            } => {
                block_stack.swap(15)?;
                (vec![], None)
            }
            Instruction {
                name: InstructionName::SWAP16,
                ..
            } => {
                block_stack.swap(16)?;
                (vec![], None)
            }

            Instruction {
                name: InstructionName::DUP1,
                ..
            } => (vec![block_stack.dup(1)?], None),
            Instruction {
                name: InstructionName::DUP2,
                ..
            } => (vec![block_stack.dup(2)?], None),
            Instruction {
                name: InstructionName::DUP3,
                ..
            } => (vec![block_stack.dup(3)?], None),
            Instruction {
                name: InstructionName::DUP4,
                ..
            } => (vec![block_stack.dup(4)?], None),
            Instruction {
                name: InstructionName::DUP5,
                ..
            } => (vec![block_stack.dup(5)?], None),
            Instruction {
                name: InstructionName::DUP6,
                ..
            } => (vec![block_stack.dup(6)?], None),
            Instruction {
                name: InstructionName::DUP7,
                ..
            } => (vec![block_stack.dup(7)?], None),
            Instruction {
                name: InstructionName::DUP8,
                ..
            } => (vec![block_stack.dup(8)?], None),
            Instruction {
                name: InstructionName::DUP9,
                ..
            } => (vec![block_stack.dup(9)?], None),
            Instruction {
                name: InstructionName::DUP10,
                ..
            } => (vec![block_stack.dup(10)?], None),
            Instruction {
                name: InstructionName::DUP11,
                ..
            } => (vec![block_stack.dup(11)?], None),
            Instruction {
                name: InstructionName::DUP12,
                ..
            } => (vec![block_stack.dup(12)?], None),
            Instruction {
                name: InstructionName::DUP13,
                ..
            } => (vec![block_stack.dup(13)?], None),
            Instruction {
                name: InstructionName::DUP14,
                ..
            } => (vec![block_stack.dup(14)?], None),
            Instruction {
                name: InstructionName::DUP15,
                ..
            } => (vec![block_stack.dup(15)?], None),
            Instruction {
                name: InstructionName::DUP16,
                ..
            } => (vec![block_stack.dup(16)?], None),

            Instruction {
                name: InstructionName::PUSH0,
                ..
            } => (vec![StackElement::Constant(num::BigUint::zero())], None),

            Instruction {
                name:
                    InstructionName::PUSH
                    | InstructionName::PUSH1
                    | InstructionName::PUSH2
                    | InstructionName::PUSH3
                    | InstructionName::PUSH4
                    | InstructionName::PUSH5
                    | InstructionName::PUSH6
                    | InstructionName::PUSH7
                    | InstructionName::PUSH8
                    | InstructionName::PUSH9
                    | InstructionName::PUSH10
                    | InstructionName::PUSH11
                    | InstructionName::PUSH12
                    | InstructionName::PUSH13
                    | InstructionName::PUSH14
                    | InstructionName::PUSH15
                    | InstructionName::PUSH16
                    | InstructionName::PUSH17
                    | InstructionName::PUSH18
                    | InstructionName::PUSH19
                    | InstructionName::PUSH20
                    | InstructionName::PUSH21
                    | InstructionName::PUSH22
                    | InstructionName::PUSH23
                    | InstructionName::PUSH24
                    | InstructionName::PUSH25
                    | InstructionName::PUSH26
                    | InstructionName::PUSH27
                    | InstructionName::PUSH28
                    | InstructionName::PUSH29
                    | InstructionName::PUSH30
                    | InstructionName::PUSH31
                    | InstructionName::PUSH32,
                value: Some(constant),
                ..
            } => (
                vec![
                    num::BigUint::from_str_radix(constant.as_str(), solx_utils::BASE_HEXADECIMAL)
                        .map(StackElement::Constant)?,
                ],
                None,
            ),
            Instruction {
                name:
                    InstructionName::PUSH_DataOffset
                    | InstructionName::PUSH_DataSize
                    | InstructionName::PUSHLIB,
                value: Some(path),
                ..
            } => (vec![StackElement::Path(path.to_owned())], None),
            Instruction {
                name: InstructionName::PUSH_Data,
                value: Some(data),
                ..
            } => (vec![StackElement::Data(data.to_owned())], None),
            instruction @ Instruction {
                name: InstructionName::PUSHDEPLOYADDRESS,
                ..
            } => (
                vec![StackElement::value(instruction.name.to_string())],
                None,
            ),

            instruction @ Instruction {
                name: InstructionName::ADD,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Constant(operand_2))
                    | (Element::Constant(operand_1), Element::Tag(operand_2))
                    | (Element::Tag(operand_1), Element::Tag(operand_2)) => {
                        match operand_1.checked_add(operand_2) {
                            Some(result)
                                if Self::is_tag_value_valid(blocks, code_segment, &result) =>
                            {
                                Element::Tag(result)
                            }
                            Some(_result) => Element::value(instruction.name.to_string()),
                            None => Element::value(instruction.name.to_string()),
                        }
                    }
                    (Element::Constant(operand_1), Element::Constant(operand_2)) => {
                        match operand_1.checked_add(operand_2) {
                            Some(result) => Element::Constant(result),
                            None => Element::value(instruction.name.to_string()),
                        }
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::SUB,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Constant(operand_2))
                    | (Element::Constant(operand_1), Element::Tag(operand_2))
                    | (Element::Tag(operand_1), Element::Tag(operand_2)) => {
                        match operand_1.checked_sub(operand_2) {
                            Some(result)
                                if Self::is_tag_value_valid(blocks, code_segment, &result) =>
                            {
                                Element::Tag(result)
                            }
                            Some(_result) => Element::value(instruction.name.to_string()),
                            None => Element::value(instruction.name.to_string()),
                        }
                    }
                    (Element::Constant(operand_1), Element::Constant(operand_2)) => {
                        match operand_1.checked_sub(operand_2) {
                            Some(result) => Element::Constant(result),
                            None => Element::value(instruction.name.to_string()),
                        }
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::MUL,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Constant(operand_2))
                    | (Element::Constant(operand_1), Element::Tag(operand_2))
                    | (Element::Tag(operand_1), Element::Tag(operand_2)) => {
                        match operand_1.checked_mul(operand_2) {
                            Some(result)
                                if Self::is_tag_value_valid(blocks, code_segment, &result) =>
                            {
                                Element::Tag(result)
                            }
                            Some(_result) => Element::value(instruction.name.to_string()),
                            None => Element::value(instruction.name.to_string()),
                        }
                    }
                    (Element::Constant(operand_1), Element::Constant(operand_2)) => {
                        match operand_1.checked_mul(operand_2) {
                            Some(result) => Element::Constant(result),
                            None => Element::value(instruction.name.to_string()),
                        }
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::DIV,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Constant(operand_2))
                    | (Element::Constant(operand_1), Element::Tag(operand_2))
                    | (Element::Tag(operand_1), Element::Tag(operand_2)) => {
                        if operand_2.is_zero() {
                            Element::Tag(num::BigUint::zero())
                        } else {
                            match operand_1.checked_div(operand_2) {
                                Some(result)
                                    if Self::is_tag_value_valid(blocks, code_segment, &result) =>
                                {
                                    Element::Tag(result)
                                }
                                Some(_result) => Element::value(instruction.name.to_string()),
                                None => Element::value(instruction.name.to_string()),
                            }
                        }
                    }
                    (Element::Constant(operand_1), Element::Constant(operand_2)) => {
                        if operand_2.is_zero() {
                            Element::Constant(num::BigUint::zero())
                        } else {
                            match operand_1.checked_div(operand_2) {
                                Some(result) => Element::Constant(result),
                                None => Element::value(instruction.name.to_string()),
                            }
                        }
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::MOD,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Constant(operand_2))
                    | (Element::Constant(operand_1), Element::Tag(operand_2))
                    | (Element::Tag(operand_1), Element::Tag(operand_2)) => {
                        if operand_2.is_zero() {
                            Element::Tag(num::BigUint::zero())
                        } else {
                            let result = operand_1 % operand_2;
                            if Self::is_tag_value_valid(blocks, code_segment, &result) {
                                Element::Tag(result)
                            } else {
                                Element::value(instruction.name.to_string())
                            }
                        }
                    }
                    (Element::Constant(operand_1), Element::Constant(operand_2)) => {
                        if operand_2.is_zero() {
                            Element::Constant(num::BigUint::zero())
                        } else {
                            Element::Constant(operand_1 % operand_2)
                        }
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::SHL,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[0], &operands[1]) {
                    (Element::Tag(tag), Element::Constant(offset)) => {
                        let offset = offset % solx_utils::BIT_LENGTH_FIELD;
                        let offset = offset.to_u64().expect("Always valid");
                        let result = tag << offset;
                        if Self::is_tag_value_valid(blocks, code_segment, &result) {
                            Element::Tag(result)
                        } else {
                            Element::value(instruction.name.to_string())
                        }
                    }
                    (Element::Constant(constant), Element::Constant(offset)) => {
                        let offset = offset % solx_utils::BIT_LENGTH_FIELD;
                        let offset = offset.to_u64().expect("Always valid");
                        Element::Constant(constant << offset)
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::SHR,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[0], &operands[1]) {
                    (Element::Tag(tag), Element::Constant(offset)) => {
                        let offset = offset % solx_utils::BIT_LENGTH_FIELD;
                        let offset = offset.to_u64().expect("Always valid");
                        let result = tag >> offset;
                        if Self::is_tag_value_valid(blocks, code_segment, &result) {
                            Element::Tag(result)
                        } else {
                            Element::value(instruction.name.to_string())
                        }
                    }
                    (Element::Constant(constant), Element::Constant(offset)) => {
                        let offset = offset % solx_utils::BIT_LENGTH_FIELD;
                        let offset = offset.to_u64().expect("Always valid");
                        Element::Constant(constant >> offset)
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::OR,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Tag(operand_2))
                    | (Element::Tag(operand_1), Element::Constant(operand_2))
                    | (Element::Constant(operand_1), Element::Tag(operand_2)) => {
                        let result = operand_1 | operand_2;
                        if Self::is_tag_value_valid(blocks, code_segment, &result) {
                            Element::Tag(result)
                        } else {
                            Element::value(instruction.name.to_string())
                        }
                    }
                    (Element::Constant(operand_1), Element::Constant(operand_2)) => {
                        Element::Constant(operand_1 | operand_2)
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::XOR,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Tag(operand_2))
                    | (Element::Tag(operand_1), Element::Constant(operand_2))
                    | (Element::Constant(operand_1), Element::Tag(operand_2)) => {
                        let result = operand_1 ^ operand_2;
                        if Self::is_tag_value_valid(blocks, code_segment, &result) {
                            Element::Tag(result)
                        } else {
                            Element::value(instruction.name.to_string())
                        }
                    }
                    (Element::Constant(operand_1), Element::Constant(operand_2)) => {
                        Element::Constant(operand_1 ^ operand_2)
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::AND,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Tag(operand_2))
                    | (Element::Tag(operand_1), Element::Constant(operand_2))
                    | (Element::Constant(operand_1), Element::Tag(operand_2)) => {
                        let result = operand_1 & operand_2;
                        if Self::is_tag_value_valid(blocks, code_segment, &result) {
                            Element::Tag(result)
                        } else {
                            Element::value(instruction.name.to_string())
                        }
                    }
                    (Element::Constant(operand_1), Element::Constant(operand_2)) => {
                        Element::Constant(operand_1 & operand_2)
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }

            instruction @ Instruction {
                name: InstructionName::LT,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Tag(operand_2)) => {
                        Element::Constant(num::BigUint::from(u64::from(operand_1 < operand_2)))
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::GT,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Tag(operand_2)) => {
                        Element::Constant(num::BigUint::from(u64::from(operand_1 > operand_2)))
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::EQ,
                ..
            } => {
                let operands = &block_stack.elements[block_stack.elements.len() - 2..];

                let result = match (&operands[1], &operands[0]) {
                    (Element::Tag(operand_1), Element::Tag(operand_2)) => {
                        Element::Constant(num::BigUint::from(u64::from(operand_1 == operand_2)))
                    }
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }
            instruction @ Instruction {
                name: InstructionName::ISZERO,
                ..
            } => {
                let operand = block_stack
                    .elements
                    .last()
                    .ok_or_else(|| anyhow::anyhow!("Operand is missing"))?;

                let result = match operand {
                    Element::Tag(operand) => Element::Constant(if operand.is_zero() {
                        num::BigUint::one()
                    } else {
                        num::BigUint::zero()
                    }),
                    _ => Element::value(instruction.name.to_string()),
                };

                (vec![result], None)
            }

            instruction => (
                vec![Element::value(instruction.name.to_string()); instruction.output_size()],
                None,
            ),
        };

        Self::update_io_data(
            block_stack,
            block_element,
            block_element.instruction.input_size(version),
            stack_output,
        )?;

        if let Some(mut queue_element) = queue_element {
            block_element.stack.clone_into(&mut queue_element.stack);
            queue.push(queue_element);
        }

        Ok(())
    }

    ///
    /// Updates the stack data with input and output data.
    ///
    fn update_io_data(
        block_stack: &mut Stack,
        block_element: &mut BlockElement,
        input_size: usize,
        output_data: Vec<Element>,
    ) -> anyhow::Result<()> {
        if block_stack.len() < input_size {
            anyhow::bail!("Stack underflow");
        }
        block_element.stack_input = Stack::new_with_elements(
            block_stack
                .elements
                .drain(block_stack.len() - input_size..)
                .collect(),
        );
        block_element.stack_output = Stack::new_with_elements(output_data);
        block_stack
            .elements
            .extend(block_element.stack_output.elements.iter().cloned());
        block_element.stack = block_stack.clone();
        Ok(())
    }

    ///
    /// Handles a defined function call.
    ///
    fn handle_function_call(
        function: &ExtraMetadataDefinedFunction,
        blocks: &HashMap<solx_codegen_evm::BlockKey, Block>,
        functions: &mut BTreeMap<solx_codegen_evm::BlockKey, Self>,
        extra_metadata: &ExtraMetadata,
        visited_functions: &mut BTreeSet<VisitedElement>,
        block_key: solx_codegen_evm::BlockKey,
        block_stack: &mut Stack,
        block_element: &mut BlockElement,
        version: &semver::Version,
    ) -> anyhow::Result<(solx_codegen_evm::BlockKey, Vec<Element>)> {
        let return_address_offset = block_stack.elements.len() - 2 - function.input_size;
        let input_arguments_offset = return_address_offset + 1;
        let callee_tag_offset = input_arguments_offset + function.input_size;

        let return_address = match block_stack.elements[return_address_offset] {
            Element::Tag(ref return_address) => {
                solx_codegen_evm::BlockKey::new(block_key.code_segment, return_address.to_owned())
            }
            ref element => anyhow::bail!("Expected the function return address, found {element}"),
        };
        let mut stack = Stack::with_capacity(1 + function.input_size);
        stack.push(StackElement::ReturnAddress(1 + function.output_size));
        stack.append(&mut Stack::new_with_elements(
            block_stack.elements[input_arguments_offset..callee_tag_offset].to_owned(),
        ));
        let stack_hash = stack.hash();

        let visited_element = VisitedElement::new(block_key.clone(), stack_hash);
        if !visited_functions.contains(&visited_element) {
            let mut function = Self::new(
                version.to_owned(),
                Some(block_key.code_segment),
                Type::new_defined(
                    function.name.to_owned(),
                    function.ast_id,
                    block_key.clone(),
                    function.input_size,
                    function.output_size,
                ),
            );
            visited_functions.insert(visited_element);
            function.traverse(blocks, functions, extra_metadata, visited_functions)?;
            functions.insert(block_key.clone(), function);
        }

        let stack_output = vec![Element::value("RETURN_VALUE".to_owned()); function.output_size];
        let mut return_stack = Stack::new_with_elements(
            block_stack.elements[..block_stack.len() - function.input_size - 2].to_owned(),
        );
        return_stack.append(&mut Stack::new_with_elements(stack_output.clone()));
        let return_stack_hash = return_stack.hash();

        block_element.instruction = Instruction::call(
            function.name.to_owned(),
            block_key,
            return_stack_hash,
            function.input_size + 2,
            function.output_size,
            return_address.clone(),
            &block_element.instruction,
        );

        Ok((return_address, stack_output))
    }

    ///
    /// Pushes a block into the function.
    ///
    fn insert_block(&mut self, mut block: Block) -> &mut Block {
        let key = block.key.clone();
        let stack_hash = block.initial_stack.hash();

        let hash_set = self.block_hash_index.entry(key.clone()).or_default();
        if hash_set.insert(stack_hash) {
            if let Some(entry) = self.blocks.get_mut(&key) {
                block.instance = Some(entry.len());
                entry.push(block);
            } else {
                block.instance = Some(0);
                self.blocks.insert(key.clone(), vec![block]);
            }
        }

        self.blocks
            .get_mut(&key)
            .expect("Always exists")
            .last_mut()
            .expect("Always exists")
    }

    ///
    /// Checks whether the tag value references an existing block in the given code segment.
    ///
    fn is_tag_value_valid(
        blocks: &HashMap<solx_codegen_evm::BlockKey, Block>,
        code_segment: solx_utils::CodeSegment,
        tag: &num::BigUint,
    ) -> bool {
        blocks.contains_key(&solx_codegen_evm::BlockKey::new(
            code_segment,
            tag & num::BigUint::from(u32::MAX),
        ))
    }

    ///
    /// Finalizes the function data.
    ///
    fn finalize(&mut self) {
        for (_tag, blocks) in self.blocks.iter() {
            for block in blocks.iter() {
                for block_element in block.elements.iter() {
                    let total_length = block_element.stack.elements.len()
                        + block_element.stack_input.len()
                        + block_element.stack_output.len();
                    if total_length > self.stack_size {
                        self.stack_size = total_length;
                    }
                }
            }
        }
    }
}

impl solx_codegen_evm::WriteLLVM for Function {
    fn declare(&mut self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        let function = match self.r#type {
            Type::Entry => context
                .get_function(self.name.as_str())
                .expect("Declared in EntryFunction"),
            Type::Defined {
                ast_id,
                input_size,
                output_size,
                ..
            } => {
                let r#type = context.function_type(
                    vec![
                        context
                            .integer_type(solx_utils::BIT_LENGTH_FIELD)
                            .as_basic_type_enum();
                        input_size
                    ],
                    output_size,
                );
                context.add_function(
                    self.name.as_str(),
                    ast_id,
                    r#type,
                    output_size,
                    Some(inkwell::module::Linkage::Private),
                )?
            }
        };
        function
            .borrow_mut()
            .set_evmla_data(solx_codegen_evm::FunctionEVMLAData::new(self.stack_size));

        Ok(())
    }

    fn into_llvm(self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        context.set_current_function(self.name.as_str())?;

        for (key, blocks) in self.blocks.iter() {
            for (index, block) in blocks.iter().enumerate() {
                let inner = context.append_basic_block(format!("block_{key}/{index}").as_str());
                let mut stack_hashes = vec![block.initial_stack.hash()];
                stack_hashes.extend_from_slice(block.extra_hashes.as_slice());
                let evmla_data = solx_codegen_evm::FunctionBlockEVMLAData::new(stack_hashes);
                let mut block = solx_codegen_evm::FunctionBlock::new(inner);
                block.set_evmla_data(evmla_data);
                context
                    .current_function()
                    .borrow_mut()
                    .evmla_mut()
                    .insert_block(key.to_owned(), block);
            }
        }

        context.set_basic_block(context.current_function().borrow().entry_block());
        let mut stack_variables = Vec::with_capacity(self.stack_size);
        for stack_index in 0..self.stack_size {
            let pointer = context.build_alloca(
                context.field_type(),
                format!("stack_var_{stack_index:03}").as_str(),
            )?;
            let value = match self.r#type {
                Type::Defined { input_size, .. }
                    if stack_index >= 1 && stack_index <= input_size =>
                {
                    context
                        .current_function()
                        .borrow()
                        .declaration()
                        .value
                        .get_nth_param((stack_index - 1) as u32)
                        .expect("Always valid")
                }
                _ => context.field_const(0).as_basic_value_enum(),
            };
            context.build_store(pointer, value)?;
            stack_variables.push(solx_codegen_evm::Value::new(
                pointer.value.as_basic_value_enum(),
            ));
        }
        context.evmla_mut().expect("Always exists").stack = stack_variables;

        match self.r#type {
            Type::Entry => {
                let initial_block = context.current_function().borrow().find_block(
                    &solx_codegen_evm::BlockKey::new(
                        context.code_segment().expect("Must be set at this point"),
                        num::BigUint::zero(),
                    ),
                    &Stack::default().hash(),
                )?;
                context.build_unconditional_branch(initial_block.inner())?;
            }
            Type::Defined { ref block_key, .. } => {
                let initial_block = context
                    .current_function()
                    .borrow()
                    .evmla()
                    .blocks
                    .get(block_key)
                    .expect("Always exists")
                    .first()
                    .expect("Always exists")
                    .inner();
                context.build_unconditional_branch(initial_block)?;
            }
        }

        for (key, blocks) in self.blocks.into_iter() {
            for (llvm_block, ir_block) in context
                .current_function()
                .borrow()
                .evmla()
                .blocks
                .get(&key)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Undeclared function block {key}"))?
                .into_iter()
                .map(|block| block.inner())
                .zip(blocks)
            {
                context.set_basic_block(llvm_block);
                ir_block.into_llvm(context)?;
            }
        }

        context.set_basic_block(context.current_function().borrow().return_block());
        match self.r#type {
            Type::Entry => {
                if !context.is_basic_block_terminated() {
                    solx_codegen_evm::r#return::stop(context)?;
                }
            }
            Type::Defined { .. } => match context.current_function().borrow().r#return() {
                solx_codegen_evm::FunctionReturn::None => {
                    context.build_return(None)?;
                }
                solx_codegen_evm::FunctionReturn::Primitive { pointer } => {
                    let return_value = context.build_load(pointer, "return_value")?;
                    context.build_return(Some(&return_value))?;
                }
                solx_codegen_evm::FunctionReturn::Compound { pointer, .. } => {
                    let return_value = context.build_load(pointer, "return_value")?;
                    context.build_return(Some(&return_value))?;
                }
            },
        }

        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.r#type {
            Type::Entry => writeln!(f, "function {} {{", self.name),
            Type::Defined {
                input_size,
                output_size,
                ..
            } => writeln!(
                f,
                "function {}({}) -> {} {{",
                self.name, input_size, output_size
            ),
        }?;
        writeln!(f, "    stack_usage: {}", self.stack_size)?;
        for (_key, blocks) in self.blocks.iter() {
            for block in blocks.iter() {
                write!(f, "{block}")?;
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
