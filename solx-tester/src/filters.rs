//!
//! `solx` tester filters.
//!

use std::collections::HashSet;

use crate::compilers::mode::Mode;

///
/// `solx` tester filters.
///
#[derive(Debug)]
pub struct Filters {
    /// The path filters.
    path_filters: HashSet<String>,
    /// The mode filters.
    mode_filters: HashSet<String>,
    /// The group filters.
    group_filters: HashSet<String>,
}

impl Filters {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        path_filters: Vec<String>,
        mode_filters: Vec<String>,
        group_filters: Vec<String>,
    ) -> Self {
        Self {
            path_filters: path_filters.into_iter().collect(),
            // Mode filters are stripped of spaces so filters like "Y+M3B3
            // 0.2.1 " and "Y +M3B3 0.2.1" become equivalent
            mode_filters: mode_filters
                .into_iter()
                .map(|f| f.replace(' ', ""))
                .collect(),
            group_filters: group_filters.into_iter().collect(),
        }
    }

    ///
    /// Check if the test path is compatible with the filters.
    ///
    pub fn check_test_path(&self, path: &str) -> bool {
        if self.path_filters.is_empty() {
            return true;
        }

        self.path_filters
            .iter()
            .any(|filter| path.contains(&filter[..filter.find("::").unwrap_or(filter.len())]))
    }

    ///
    /// Check if the test case path is compatible with the filters.
    ///
    pub fn check_case_path(&self, path: &str) -> bool {
        self.path_filters.is_empty() || self.path_filters.iter().any(|filter| path.contains(filter))
    }

    ///
    /// Check if the mode is compatible with the filters.
    ///
    pub fn check_mode(&self, mode: &Mode) -> bool {
        mode.check_filters(&self.mode_filters)
    }

    ///
    /// Check if the test group is compatible with the filters.
    ///
    pub fn check_group(&self, group: &Option<String>) -> bool {
        if self.group_filters.is_empty() {
            return true;
        }

        if let Some(group) = group {
            !self.group_filters.contains(group)
        } else {
            false
        }
    }
}
