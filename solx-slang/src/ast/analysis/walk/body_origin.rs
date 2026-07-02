//!
//! Whether a walked function body originates in a contract or a library.
//!

/// Where the function body being walked originates. Gates the library-call collector's bare-identifier
/// sibling collection: only library bodies reach no-selector siblings by bare name.
#[derive(Default)]
pub enum BodyOrigin {
    /// A contract's own, inherited, or free function body.
    #[default]
    Contract,
    /// A library function body.
    Library,
}
