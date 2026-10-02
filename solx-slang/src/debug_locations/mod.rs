//!
//! Source locations for debug info: the source range of each node, the compilation's sources, and
//! the location the ops lowered from each construct carry.
//!

pub mod resolver;
pub mod sources;

use melior::ir::Location;
use slang_solidity_v2::ast::NodeLocation;

use self::resolver::Resolver;
use self::sources::Sources;

/// What one object's nodes resolve to: their locations when the object requested debug info, and
/// otherwise the unknown location, with no source range read.
pub enum DebugLocations<'context> {
    /// Without debug info: every node resolves to the unknown location, built once.
    Disabled(Location<'context>),
    /// With debug info: every node resolves to its location.
    Enabled(Resolver<'context>),
}

impl<'context> DebugLocations<'context> {
    /// Resolves nodes through `sources` in `melior`, or to the unknown location when `sources` is
    /// absent.
    pub fn new(
        melior: &'context melior::Context,
        sources: Option<&'context Sources<'context>>,
    ) -> Self {
        match sources {
            Some(sources) => Self::Enabled(Resolver::new(melior, sources)),
            None => Self::Disabled(Location::unknown(melior)),
        }
    }

    /// The location of `node`'s first byte, which the ops lowered from it carry.
    pub fn location(&mut self, node: &impl NodeLocation) -> Location<'context> {
        match self {
            Self::Disabled(location) => *location,
            Self::Enabled(resolver) => resolver.location(node),
        }
    }

    /// The location of `node`'s last byte, a body's closing brace.
    pub fn location_end(&mut self, node: &impl NodeLocation) -> Location<'context> {
        match self {
            Self::Disabled(location) => *location,
            Self::Enabled(resolver) => resolver.location_end(node),
        }
    }
}
