//!
//! Compiler pipeline profiler.
//!

pub mod run;

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::optimizer::settings::Settings as OptimizerSettings;

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
        let run_name = description.to_owned();
        assert!(
            !self.timings.contains_key(run_name.as_str()),
            "Translation unit run `{run_name}` already exists"
        );

        self.start_run(run_name)
    }

    ///
    /// Starts a new run for an EVM translation unit.
    ///
    pub fn start_evm_translation_unit(
        &mut self,
        full_path: &str,
        code_segment: solx_utils::CodeSegment,
        description: &str,
        optimizer_settings: &OptimizerSettings,
    ) -> Rc<RefCell<Run>> {
        let run_name = format!("{full_path}:{code_segment}/{description}/{optimizer_settings}",);
        assert!(
            !self.timings.contains_key(run_name.as_str()),
            "Translation unit run `{run_name}` already exists"
        );

        self.start_run(run_name)
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
                    run.duration.expect("Always exists").as_millis() as u64,
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
