//!
//! Movements collected for the inline "largest changes" listings.
//!

use std::cmp::Reverse;

///
/// Movements collected for the inline "largest changes" listings.
///
#[derive(Default)]
pub struct TopMovers(Vec<Movement>);

///
/// One row's movement between the main and PR toolchains.
///
pub struct Movement {
    /// The row label: the project or case that moved.
    pub label: String,
    /// The humanized toolchain the movement was measured on.
    pub mode: String,
    /// The `main` baseline value.
    pub main: u64,
    /// The PR value.
    pub pr: u64,
}

impl TopMovers {
    /// Records a movement for the inline listing.
    pub fn push(&mut self, label: &str, mode: &str, main: u64, pr: u64) {
        self.0.push(Movement {
            label: label.to_owned(),
            mode: mode.to_owned(),
            main,
            pr,
        });
    }

    /// The movements ordered by descending magnitude, so the renderer lists
    /// the biggest first and counts the rest as "+N more".
    pub fn ranked(&self) -> Vec<&Movement> {
        let mut movers: Vec<&Movement> = self.0.iter().collect();
        movers.sort_by_key(|movement| {
            Reverse((movement.pr as i128 - movement.main as i128).unsigned_abs())
        });
        movers
    }

    /// Whether no movement was recorded, so the listing is skipped.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
