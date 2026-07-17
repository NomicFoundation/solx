//!
//! Movements collected for the inline "largest changes" listings.
//!

use crate::output::summary::movement::Movement;

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
            std::cmp::Reverse((movement.pr as i128 - movement.main as i128).unsigned_abs())
        });
        movers
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::output::summary::top_movers::TopMovers;

    #[test]
    fn movers_rank_by_magnitude_regardless_of_direction() {
        let mut movers = TopMovers::default();
        movers.push("small", "legacy", 100, 103);
        movers.push("shrunk", "legacy", 100, 80);
        movers.push("grown", "legacy", 100, 110);
        let ranked = movers.ranked();
        let labels: Vec<&str> = ranked
            .iter()
            .map(|movement| movement.label.as_str())
            .collect();
        assert_eq!(labels, ["shrunk", "grown", "small"]);
    }
}
