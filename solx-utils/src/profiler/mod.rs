//!
//! Compiler pipeline profiler.
//!

pub mod run;

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::CodeSegment;

use self::run::Run;

///
/// Compiler pipeline profiler.
///
#[derive(Debug, Default)]
pub struct Profiler {
    /// Indexed map of timing entries.
    pub timings: IndexMap<String, Rc<RefCell<Run>>>,
}

impl Profiler {
    ///
    /// Starts a new run for a generic part of the pipeline.
    ///
    pub fn start_pipeline_element(&mut self, description: &str) -> Rc<RefCell<Run>> {
        self.start_run(description.to_owned())
    }

    ///
    /// Starts a new run for an EVM translation unit.
    ///
    pub fn start_evm_translation_unit(
        &mut self,
        full_path: &str,
        code_segment: CodeSegment,
        description: &str,
        optimizer_mode: &str,
        spill_area_size: Option<u64>,
    ) -> Rc<RefCell<Run>> {
        self.start_run(format!(
            "{full_path}:{code_segment}/{description}/{optimizer_mode}/SpillArea({})",
            spill_area_size.unwrap_or_default()
        ))
    }

    ///
    /// Returns a serializeable vector of the profiler runs.
    ///
    pub fn to_vec(&self) -> Vec<(String, u64)> {
        self.timings
            .iter()
            .map(|(name, run)| {
                let run = run.borrow();
                (
                    name.clone(),
                    run.duration.expect("Always exists").as_micros() as u64,
                )
            })
            .collect()
    }

    ///
    /// Starts a new run with the given name.
    ///
    fn start_run(&mut self, name: String) -> Rc<RefCell<Run>> {
        assert!(
            !self.timings.contains_key(name.as_str()),
            "Run `{name}` already exists"
        );

        let run = Rc::new(RefCell::new(Run::default()));
        self.timings.insert(name, run.clone());
        run
    }
}
