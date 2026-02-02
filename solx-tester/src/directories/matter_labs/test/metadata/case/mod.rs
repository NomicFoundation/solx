//!
//! The Matter Labs compiler test metadata case.
//!

pub mod input;

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::compilers::mode::Mode;
use crate::revm::address_iterator::AddressIterator;
use crate::test::case::input::value::Value;
use crate::test::instance::Instance;

use self::input::Input;
use self::input::expected::Expected;

///
/// The Matter Labs compiler test metadata case.
///
#[derive(Debug, Clone, Deserialize)]
pub struct Case {
    /// The comment to a case.
    pub comment: Option<String>,
    /// The case name.
    pub name: String,
    /// The mode filter.
    pub modes: Option<Vec<String>>,
    /// The case inputs.
    pub inputs: Vec<Input>,
    /// If the test case must be ignored.
    #[serde(default)]
    pub ignore: bool,
    /// Overrides the default number of cycles.
    pub cycles: Option<usize>,

    /// The expected return data.
    pub expected: Option<Expected>,
}

impl Case {
    ///
    /// Normalizes the case.
    ///
    pub fn normalize(
        mut self,
        contracts: &BTreeMap<String, String>,
        instances: &BTreeMap<String, Instance>,
    ) -> anyhow::Result<Self> {
        self.normalize_deployer_calls(contracts, instances)?;
        self.normalize_expected();
        Ok(self)
    }

    ///
    /// Validates deployer calls, adds libraries deployer calls, contracts deployer calls if they are not present.
    ///
    pub fn normalize_deployer_calls(
        &mut self,
        contracts: &BTreeMap<String, String>,
        instances: &BTreeMap<String, Instance>,
    ) -> anyhow::Result<()> {
        let mut contracts = contracts.clone();
        for (index, input) in self.inputs.iter().enumerate() {
            if input.method.as_str() != "#deployer" {
                continue;
            };

            if contracts.remove(input.instance.as_str()).is_none() {
                anyhow::bail!(
                    "Input {index} is a second deployer call for the same instance or instance is invalid"
                );
            }
        }

        let mut inputs = Vec::with_capacity(instances.len() + self.inputs.len());

        for (name, instance) in instances.iter() {
            if instance.is_library() {
                inputs.push(Input::empty_deployer_call(name.to_owned()));
            }
        }

        for contract in contracts.keys() {
            if !instances
                .iter()
                .any(|(filter_name, instance)| filter_name == contract && instance.is_library())
            {
                inputs.push(Input::empty_deployer_call(contract.clone()));
            }
        }

        inputs.append(&mut self.inputs);
        self.inputs = inputs;

        Ok(())
    }

    ///
    /// Copies the final expected data to the last input.
    ///
    pub fn normalize_expected(&mut self) {
        if let Some(input) = self.inputs.last_mut()
            && input.expected.is_none()
        {
            input.expected.clone_from(&self.expected);
        }
    }

    ///
    /// Sets all variables, including instance addresses, but except libraries.
    ///
    pub fn set_variables(
        &self,
        instances: &mut BTreeMap<String, Instance>,
        mut address_iterator: AddressIterator,
        mode: &Mode,
    ) -> anyhow::Result<()> {
        for (index, input) in self.inputs.iter().enumerate() {
            if input.method.as_str() != "#deployer"
                || instances.iter().any(|(name, instance)| {
                    name.as_str() == input.instance.as_str() && instance.is_library()
                })
            {
                continue;
            }

            let exception = match input.expected.as_ref() {
                Some(expected) => expected
                    .exception(mode)
                    .map_err(|error| anyhow::anyhow!("Input #{index}: {error}"))?,
                None => false,
            };
            if exception {
                continue;
            }

            let caller = match Value::try_from_matter_labs(input.caller.as_str(), instances)
                .map_err(|error| anyhow::anyhow!("Invalid caller `{}`: {error}", input.caller))?
            {
                Value::Known(value) => crate::utils::u256_to_address(&value),
                Value::Any => anyhow::bail!("Caller can not be `*`"),
            };

            if let Some(instance) = instances.get_mut(input.instance.as_str()) {
                instance.set_address(address_iterator.next(&caller, true));
            }
        }
        Ok(())
    }
}
