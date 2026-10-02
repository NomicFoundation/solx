//!
//! The name of a source file, as the attribute its locations reference.
//!

use melior::ir::Attribute;
use melior::ir::AttributeLike;
use melior::ir::Location;
use melior::ir::attribute::StringAttribute;

/// The name of a source file, built once for every location in the file: a location built from
/// a path uniques the path again each time.
#[derive(Clone, Copy)]
pub struct DebugInfoFileName<'context> {
    /// The string attribute every location in the file references.
    attribute: Attribute<'context>,
}

impl<'context> DebugInfoFileName<'context> {
    /// Uniques `name` in `melior`.
    pub fn new(melior: &'context melior::Context, name: &str) -> Self {
        Self {
            attribute: StringAttribute::new(melior, name).into(),
        }
    }

    /// The location of `line` and `column`, both 1-based, in the file.
    pub fn location(self, line: usize, column: usize) -> Location<'context> {
        unsafe {
            Location::from_raw(crate::ffi::solxCreateFileLineColLoc(
                self.attribute.to_raw(),
                line as u32,
                column as u32,
            ))
        }
    }
}
