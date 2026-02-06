//!
//! The Ethereal IR block element stack.
//!

pub mod element;

use std::hash::Hasher;

use twox_hash::XxHash3_64;

use self::element::Element;

///
/// The Ethereal IR block element stack.
///
#[derive(Debug, Default, Clone)]
pub struct Stack {
    /// The stack elements.
    pub elements: Vec<Element>,
}

impl Stack {
    /// The default stack size.
    pub const DEFAULT_STACK_SIZE: usize = 16;

    ///
    /// A shortcut constructor.
    ///
    pub fn new() -> Self {
        Self {
            elements: Vec::with_capacity(Self::DEFAULT_STACK_SIZE),
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn new_with_elements(elements: Vec<Element>) -> Self {
        Self { elements }
    }

    ///
    /// The stack state hash, which acts as a block identifier.
    ///
    /// Each block clone has its own initial stack state, which uniquely identifies the block.
    ///
    pub fn hash(&self) -> u64 {
        let mut hasher = XxHash3_64::default();
        for element in self.elements.iter() {
            match element {
                Element::Tag(tag) => hasher.write(tag.to_bytes_le().as_slice()),
                _ => hasher.write_u8(0),
            }
        }
        hasher.finish()
    }

    ///
    /// Pushes an element onto the stack.
    ///
    pub fn push(&mut self, element: Element) {
        self.elements.push(element);
    }

    ///
    /// Appends another stack on top of this one, consuming its elements.
    ///
    pub fn append(&mut self, other: &mut Self) {
        self.elements.append(&mut other.elements);
    }

    ///
    /// Extends this stack by cloning elements from another stack.
    ///
    pub fn extend_from(&mut self, other: &Self) {
        self.elements.extend(other.elements.iter().cloned());
    }

    ///
    /// Pops a stack element.
    ///
    pub fn pop(&mut self) -> anyhow::Result<Element> {
        self.elements
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Stack underflow"))
    }

    ///
    /// Pops the tag from the top.
    ///
    pub fn pop_tag(&mut self) -> anyhow::Result<num::BigUint> {
        match self.elements.pop() {
            Some(Element::Tag(tag)) => Ok(tag),
            Some(element) => anyhow::bail!("Expected tag, found {element}"),
            None => anyhow::bail!("Stack underflow"),
        }
    }

    ///
    /// Pops the constant from the top.
    ///
    pub fn pop_constant(&mut self) -> anyhow::Result<num::BigUint> {
        match self.elements.pop() {
            Some(Element::Constant(constant)) => Ok(constant),
            Some(element) => anyhow::bail!("Expected constant, found {element}"),
            None => anyhow::bail!("Stack underflow"),
        }
    }

    ///
    /// Swaps two stack elements.
    ///
    pub fn swap(&mut self, index: usize) -> anyhow::Result<()> {
        if self.elements.len() < index + 1 {
            anyhow::bail!("Stack underflow");
        }

        let length = self.elements.len();
        self.elements.swap(length - 1, length - 1 - index);

        Ok(())
    }

    ///
    /// Duplicates a stack element.
    ///
    pub fn dup(&mut self, index: usize) -> anyhow::Result<Element> {
        if self.elements.len() < index {
            anyhow::bail!("Stack underflow");
        }

        Ok(self.elements[self.elements.len() - index].to_owned())
    }

    ///
    /// Returns the stack length.
    ///
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    ///
    /// Returns an emptiness flag.
    ///
    pub fn is_empty(&self) -> bool {
        self.elements.len() == 0
    }
}

impl std::fmt::Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {} ]",
            self.elements
                .iter()
                .map(Element::to_string)
                .collect::<Vec<String>>()
                .join(" | ")
        )
    }
}
