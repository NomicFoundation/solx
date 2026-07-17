//!
//! Movements collected for the inline "largest changes" listings.
//!

use std::cmp::Reverse;

///
/// Movements collected for the inline "largest changes" listings.
///
#[derive(Default)]
pub struct TopMovers(Vec<Movement>);

impl TopMovers {
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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

///
/// One row's movement between the main and PR toolchains.
///
pub struct Movement {
    pub label: String,
    pub mode: String,
    pub main: u64,
    pub pr: u64,
}
