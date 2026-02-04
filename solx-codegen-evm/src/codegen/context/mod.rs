//!
//! The LLVM IR generator context.
//!

pub mod address_space;
pub mod evmla_data;
pub mod function;
pub mod solidity_data;
pub mod yul_data;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::ISolidityData;
use crate::codegen::build::Build as EVMBuild;
use crate::codegen::profiler::Profiler;
use crate::codegen::warning::Warning;
use crate::context::IContext;
use crate::context::attribute::Attribute;
use crate::context::debug_info::DebugInfo;
use crate::context::function::declaration::Declaration as FunctionDeclaration;
use crate::context::function::r#return::Return as FunctionReturn;
use crate::context::r#loop::Loop;
use crate::debug_config::OutputConfig;
use crate::optimizer::Optimizer;
use crate::optimizer::settings::Settings as OptimizerSettings;
use crate::target_machine::TargetMachine;

use self::address_space::AddressSpace;
use self::evmla_data::EVMLAData;
use self::function::Function;
use self::function::intrinsics::Intrinsics;
use self::solidity_data::SolidityData;
use self::yul_data::YulData;

///
/// The LLVM IR generator context.
///
/// It is a not-so-big god-like object glueing all the compilers' complexity and act as an adapter
/// and a superstructure over the inner `inkwell` LLVM context.
///
pub struct Context<'ctx> {
    /// The inner LLVM context.
    llvm: &'ctx inkwell::context::Context,
    /// The inner LLVM context builder.
    builder: inkwell::builder::Builder<'ctx>,
    /// The optimization tools.
    optimizer: Optimizer,
    /// The current module.
    module: inkwell::module::Module<'ctx>,
    /// The extra LLVM options.
    llvm_options: Vec<String>,
    /// The full contract name.
    contract_name: solx_utils::ContractName,
    /// The current contract code type, which can be deploy or runtime.
    code_segment: solx_utils::CodeSegment,
    /// The EVM version to produce bytecode for.
    evm_version: Option<solx_utils::EVMVersion>,
    /// The LLVM intrinsic functions, defined on the LLVM side.
    intrinsics: Intrinsics<'ctx>,
    /// The declared functions.
    functions: HashMap<String, Rc<RefCell<Function<'ctx>>>>,
    /// The current active function.
    current_function: Option<Rc<RefCell<Function<'ctx>>>>,
    /// The loop context stack.
    loop_stack: Vec<Loop<'ctx>>,

    /// The debug info of the current module.
    debug_info: Option<DebugInfo<'ctx>>,
    /// The output configuration telling whether to dump the needed IRs.
    output_config: Option<OutputConfig>,

    /// The Solidity data.
    solidity_data: Option<SolidityData>,
    /// The Yul data.
    yul_data: Option<YulData>,
    /// The EVM legacy assembly data.
    evmla_data: Option<EVMLAData<'ctx>>,

    /// Captured EVM legacy assembly IR for output.
    captured_evmla: Option<String>,
    /// Captured Ethereal IR for output.
    captured_ethir: Option<String>,
    /// Whether to capture LLVM IR for output.
    capture_llvm_ir: bool,
}

impl<'ctx> Context<'ctx> {
    /// The functions hashmap default capacity.
    const FUNCTIONS_HASHMAP_INITIAL_CAPACITY: usize = 64;

    /// The loop stack default capacity.
    const LOOP_STACK_INITIAL_CAPACITY: usize = 16;

    ///
    /// Initializes a new LLVM context.
    ///
    pub fn new(
        llvm: &'ctx inkwell::context::Context,
        module: inkwell::module::Module<'ctx>,
        llvm_options: Vec<String>,
        contract_name: solx_utils::ContractName,
        code_segment: solx_utils::CodeSegment,
        evm_version: Option<solx_utils::EVMVersion>,
        optimizer: Optimizer,
        output_debug_info: bool,
        solidity_data: Option<SolidityData>,
        output_config: Option<OutputConfig>,
    ) -> Self {
        let builder = llvm.create_builder();
        let intrinsics = Intrinsics::new(llvm, &module);
        let debug_info = if output_debug_info {
            solidity_data
                .as_ref()
                .filter(|solidity_data| solidity_data.debug_info().is_some())
                .map(|solidity_data| solidity_data.sources())
                .map(|sources| DebugInfo::new(&module, sources))
        } else {
            None
        };

        Self {
            llvm,
            builder,
            llvm_options,
            optimizer,
            module,
            contract_name,
            code_segment,
            evm_version,
            intrinsics,
            functions: HashMap::with_capacity(Self::FUNCTIONS_HASHMAP_INITIAL_CAPACITY),
            current_function: None,
            loop_stack: Vec::with_capacity(Self::LOOP_STACK_INITIAL_CAPACITY),

            debug_info,
            output_config,

            solidity_data,
            yul_data: None,
            evmla_data: None,

            captured_evmla: None,
            captured_ethir: None,
            capture_llvm_ir: false,
        }
    }

    ///
    /// Builds the LLVM IR module, returning the build artifacts.
    ///
    pub fn build(
        &mut self,
        output_assembly: bool,
        output_bytecode: bool,
        is_size_fallback: bool,
        profiler: &mut Profiler,
    ) -> anyhow::Result<EVMBuild> {
        let contract_path = self.module.get_name().to_str().expect("Always valid");

        let run_init_verify = profiler.start_evm_translation_unit(
            contract_path,
            self.code_segment,
            "InitVerify",
            self.optimizer.settings(),
        );
        let target_machine =
            TargetMachine::new(self.optimizer.settings(), self.llvm_options.as_slice())?;
        target_machine.set_target_data(self.module());
        target_machine.set_asm_verbosity(true);

        if let Some(debug_info) = self.debug_info.take() {
            debug_info.finalize(self);
            self.debug_info = Some(debug_info);
        }

        let spill_area = self
            .optimizer
            .settings()
            .spill_area_size()
            .map(|spill_area_size| (crate::r#const::SOLC_USER_MEMORY_OFFSET, spill_area_size));

        if let Some(output_config) = self.output_config.as_ref() {
            output_config.dump_llvm_ir_unoptimized(
                contract_path,
                self.module(),
                is_size_fallback,
                spill_area,
            )?;
        }

        // Capture unoptimized LLVM IR for output if requested and not writing to files
        let captured_llvm_ir = if self.capture_llvm_ir && self.output_config.is_none() {
            Some(self.module().print_to_string().to_string())
        } else {
            None
        };

        self.verify().map_err(|error| {
            anyhow::anyhow!(
                "{} code unoptimized LLVM IR verification: {error}",
                self.code_segment,
            )
        })?;
        run_init_verify.borrow_mut().finish();

        let module_size_fallback = self.module.clone();
        let run_optimize_verify = profiler.start_evm_translation_unit(
            contract_path,
            self.code_segment,
            "OptimizeVerify",
            self.optimizer.settings(),
        );
        self.optimizer
            .run(&target_machine, self.module())
            .map_err(|error| anyhow::anyhow!("{} code optimizing: {error}", self.code_segment))?;
        if let Some(output_config) = self.output_config.as_ref() {
            output_config.dump_llvm_ir_optimized(
                contract_path,
                self.module(),
                is_size_fallback,
                spill_area,
            )?;
        }

        // Capture optimized LLVM IR for output if requested and not writing to files
        let captured_llvm_ir_optimized = if self.capture_llvm_ir && self.output_config.is_none() {
            Some(self.module().print_to_string().to_string())
        } else {
            None
        };

        self.verify().map_err(|error| {
            anyhow::anyhow!(
                "{} code optimized LLVM IR verification: {error}",
                self.code_segment,
            )
        })?;
        run_optimize_verify.borrow_mut().finish();

        let assembly_buffer = if output_assembly || self.output_config.is_some() {
            let run_emit_llvm_assembly = profiler.start_evm_translation_unit(
                contract_path,
                self.code_segment,
                "EmitLLVMAssembly",
                self.optimizer.settings(),
            );
            let module_assembly_emitter = self.module.clone();
            let assembly_buffer = target_machine
                .write_to_memory_buffer(
                    &module_assembly_emitter,
                    inkwell::targets::FileType::Assembly,
                )
                .map_err(|error| anyhow::anyhow!("assembly emitting: {error}"))?;

            if let Some(output_config) = self.output_config.as_ref() {
                let assembly_text = String::from_utf8_lossy(assembly_buffer.as_slice());
                output_config.dump_assembly(
                    contract_path,
                    assembly_text.as_ref(),
                    is_size_fallback,
                    spill_area,
                )?;
            }

            run_emit_llvm_assembly.borrow_mut().finish();
            Some(assembly_buffer)
        } else {
            None
        };
        let assembly = assembly_buffer
            .map(|assembly_buffer| String::from_utf8_lossy(assembly_buffer.as_slice()).to_string());

        if output_bytecode || self.debug_info.is_some() {
            let run_emit_bytecode = profiler.start_evm_translation_unit(
                contract_path,
                self.code_segment,
                "EmitBytecode",
                self.optimizer.settings(),
            );
            let (bytecode_buffer, debug_info_buffer) = if self.debug_info.is_some() {
                let (bytecode_buffer, debug_info_buffer) = target_machine
                    .write_to_memory_buffer_with_debug_info(
                        self.module(),
                        inkwell::targets::FileType::Object,
                    )
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "{} bytecode and debug info emitting: {error}",
                            self.code_segment
                        )
                    })?;
                (bytecode_buffer, Some(debug_info_buffer))
            } else {
                let bytecode_buffer = target_machine
                    .write_to_memory_buffer(self.module(), inkwell::targets::FileType::Object)
                    .map_err(|error| {
                        anyhow::anyhow!("{} bytecode emitting: {error}", self.code_segment)
                    })?;
                (bytecode_buffer, None)
            };
            run_emit_bytecode.borrow_mut().finish();

            let immutables = match self.code_segment {
                solx_utils::CodeSegment::Deploy => None,
                solx_utils::CodeSegment::Runtime => Some(bytecode_buffer.get_immutables_evm()),
            };

            let bytecode_size_limit = match self.code_segment {
                solx_utils::CodeSegment::Deploy => crate::r#const::DEPLOY_CODE_SIZE_LIMIT,
                solx_utils::CodeSegment::Runtime => crate::r#const::RUNTIME_CODE_SIZE_LIMIT,
            };

            let mut warnings = Vec::with_capacity(1);
            let bytecode_size = bytecode_buffer.as_slice().len();
            if bytecode_size > bytecode_size_limit {
                if self.optimizer.settings() == &OptimizerSettings::cycles()
                    && self.optimizer.settings().is_fallback_to_size_enabled()
                {
                    crate::codegen::IS_SIZE_FALLBACK
                        .compare_exchange(
                            false,
                            true,
                            std::sync::atomic::Ordering::Relaxed,
                            std::sync::atomic::Ordering::Relaxed,
                        )
                        .expect("Failed to set the global size fallback flag");
                    self.optimizer = Optimizer::new(OptimizerSettings::size());
                    self.module = module_size_fallback;
                    for function in self.module.get_functions() {
                        Function::set_size_attributes(self.llvm, function);
                    }
                    return self.build(output_assembly, output_bytecode, true, profiler);
                } else {
                    warnings.push(match self.code_segment {
                        solx_utils::CodeSegment::Deploy => Warning::DeployCodeSize {
                            found: bytecode_size,
                        },
                        solx_utils::CodeSegment::Runtime => Warning::RuntimeCodeSize {
                            found: bytecode_size,
                        },
                    })
                };
            }
            // Only capture EVMLA/EthIR if not writing to files
            let captured_evmla = if self.output_config.is_none() {
                self.captured_evmla.take()
            } else {
                None
            };
            let captured_ethir = if self.output_config.is_none() {
                self.captured_ethir.take()
            } else {
                None
            };
            Ok(EVMBuild::new(
                Some(bytecode_buffer.as_slice().to_vec()),
                debug_info_buffer.map(|buffer| buffer.as_slice().to_vec()),
                assembly,
                captured_evmla,
                captured_ethir,
                captured_llvm_ir,
                captured_llvm_ir_optimized,
                immutables,
                is_size_fallback,
                warnings,
            ))
        } else {
            // Only capture EVMLA/EthIR if not writing to files
            let captured_evmla = if self.output_config.is_none() {
                self.captured_evmla.take()
            } else {
                None
            };
            let captured_ethir = if self.output_config.is_none() {
                self.captured_ethir.take()
            } else {
                None
            };
            Ok(EVMBuild::new(
                None,
                None,
                assembly,
                captured_evmla,
                captured_ethir,
                captured_llvm_ir,
                captured_llvm_ir_optimized,
                None,
                is_size_fallback,
                vec![],
            ))
        }
    }

    ///
    /// Verifies the current LLVM IR module.
    ///
    pub fn verify(&self) -> anyhow::Result<()> {
        self.module()
            .verify()
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }

    ///
    /// Sets the captured EVM legacy assembly IR.
    ///
    pub fn set_captured_evmla(&mut self, evmla: String) {
        self.captured_evmla = Some(evmla);
    }

    ///
    /// Sets the captured Ethereal IR.
    ///
    pub fn set_captured_ethir(&mut self, ethir: String) {
        self.captured_ethir = Some(ethir);
    }

    ///
    /// Enables LLVM IR capture for output.
    ///
    pub fn set_capture_llvm_ir(&mut self, capture: bool) {
        self.capture_llvm_ir = capture;
    }

    ///
    /// Takes the captured EVM legacy assembly IR.
    ///
    pub fn take_captured_evmla(&mut self) -> Option<String> {
        self.captured_evmla.take()
    }

    ///
    /// Takes the captured Ethereal IR.
    ///
    pub fn take_captured_ethir(&mut self) -> Option<String> {
        self.captured_ethir.take()
    }

    ///
    /// Returns the LLVM intrinsics collection reference.
    ///
    pub fn intrinsics(&self) -> &Intrinsics<'ctx> {
        &self.intrinsics
    }

    ///
    /// Returns a Yul function type with the specified arguments and number of return values.
    ///
    pub fn function_type<T>(
        &self,
        argument_types: Vec<T>,
        return_values_size: usize,
    ) -> inkwell::types::FunctionType<'ctx>
    where
        T: BasicType<'ctx>,
    {
        let argument_types: Vec<inkwell::types::BasicMetadataTypeEnum> = argument_types
            .as_slice()
            .iter()
            .map(T::as_basic_type_enum)
            .map(inkwell::types::BasicMetadataTypeEnum::from)
            .collect();
        match return_values_size {
            0 => self
                .llvm
                .void_type()
                .fn_type(argument_types.as_slice(), false),
            1 => self.field_type().fn_type(argument_types.as_slice(), false),
            size => self
                .structure_type(vec![self.field_type().as_basic_type_enum(); size].as_slice())
                .fn_type(argument_types.as_slice(), false),
        }
    }

    ///
    /// Modifies the call site value, setting the default attributes.
    ///
    /// The attributes only affect the LLVM optimizations.
    ///
    pub fn modify_call_site_value(
        &self,
        arguments: &[inkwell::values::BasicMetadataValueEnum<'ctx>],
        call_site_value: inkwell::values::CallSiteValue<'ctx>,
        function: FunctionDeclaration<'ctx>,
    ) {
        for (index, argument) in arguments.iter().enumerate() {
            if argument.is_pointer_value() {
                call_site_value.set_alignment_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    solx_utils::BYTE_LENGTH_FIELD as u32,
                );
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm
                        .create_enum_attribute(Attribute::NoAlias as u32, 0),
                );
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm
                        .create_enum_attribute(Attribute::Captures as u32, 0),
                );
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm.create_enum_attribute(Attribute::NoFree as u32, 0),
                );
                if (*argument)
                    .try_into()
                    .map(|argument: inkwell::values::BasicValueEnum<'ctx>| argument.get_type())
                    .ok()
                    == function.r#type.get_return_type()
                {
                    if function
                        .r#type
                        .get_return_type()
                        .map(|r#type| {
                            r#type.into_pointer_type().get_address_space()
                                == AddressSpace::Stack.into()
                        })
                        .unwrap_or_default()
                    {
                        call_site_value.add_attribute(
                            inkwell::attributes::AttributeLoc::Param(index as u32),
                            self.llvm
                                .create_enum_attribute(Attribute::Returned as u32, 0),
                        );
                    }
                    call_site_value.add_attribute(
                        inkwell::attributes::AttributeLoc::Param(index as u32),
                        self.llvm.create_enum_attribute(
                            Attribute::Dereferenceable as u32,
                            (solx_utils::BIT_LENGTH_FIELD * 2) as u64,
                        ),
                    );
                    call_site_value.add_attribute(
                        inkwell::attributes::AttributeLoc::Return,
                        self.llvm.create_enum_attribute(
                            Attribute::Dereferenceable as u32,
                            (solx_utils::BIT_LENGTH_FIELD * 2) as u64,
                        ),
                    );
                }
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm
                        .create_enum_attribute(Attribute::NonNull as u32, 0),
                );
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm
                        .create_enum_attribute(Attribute::NoUndef as u32, 0),
                );
            }
        }

        if function
            .r#type
            .get_return_type()
            .map(|r#type| r#type.is_pointer_type())
            .unwrap_or_default()
        {
            call_site_value.set_alignment_attribute(
                inkwell::attributes::AttributeLoc::Return,
                solx_utils::BYTE_LENGTH_FIELD as u32,
            );
            call_site_value.add_attribute(
                inkwell::attributes::AttributeLoc::Return,
                self.llvm
                    .create_enum_attribute(Attribute::NoAlias as u32, 0),
            );
            call_site_value.add_attribute(
                inkwell::attributes::AttributeLoc::Return,
                self.llvm
                    .create_enum_attribute(Attribute::NonNull as u32, 0),
            );
            call_site_value.add_attribute(
                inkwell::attributes::AttributeLoc::Return,
                self.llvm
                    .create_enum_attribute(Attribute::NoUndef as u32, 0),
            );
        }
    }
}

impl<'ctx> IContext<'ctx> for Context<'ctx> {
    type Function = Function<'ctx>;

    type AddressSpace = AddressSpace;

    type SolidityData = SolidityData;

    type YulData = YulData;

    type EVMLAData = EVMLAData<'ctx>;

    fn llvm(&self) -> &'ctx inkwell::context::Context {
        self.llvm
    }

    fn builder(&self) -> &inkwell::builder::Builder<'ctx> {
        &self.builder
    }

    fn module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    fn optimizer(&self) -> &Optimizer {
        &self.optimizer
    }

    fn debug_info(&self) -> Option<&DebugInfo<'ctx>> {
        self.debug_info.as_ref()
    }

    fn create_debug_info_location(&self) -> Option<inkwell::debug_info::DILocation<'ctx>> {
        let debug_info = self.debug_info.as_ref()?;
        let current_function = self
            .current_function
            .as_ref()
            .expect("Always exists")
            .borrow();
        let current_location = self
            .solidity()
            .and_then(|solidity_data| solidity_data.get_solx_location())
            .or(current_function.solx_debug_info_location())
            .cloned()
            .unwrap_or_else(|| {
                solx_utils::DebugInfoMappedLocation::new_with_location(
                    self.contract_name.path.to_owned(),
                    1,
                    1,
                    0,
                    None,
                )
            });
        debug_info.create_location(
            self,
            current_location.line.unwrap_or_default(),
            current_location.column.unwrap_or_default(),
        )
    }

    fn output_config(&self) -> Option<&OutputConfig> {
        self.output_config.as_ref()
    }

    fn contract_name(&self) -> &solx_utils::ContractName {
        &self.contract_name
    }

    fn set_code_segment(&mut self, code_segment: solx_utils::CodeSegment) {
        self.code_segment = code_segment;
    }

    fn code_segment(&self) -> Option<solx_utils::CodeSegment> {
        Some(self.code_segment.to_owned())
    }

    fn evm_version(&self) -> solx_utils::EVMVersion {
        self.evm_version.unwrap_or_default()
    }

    fn append_basic_block(&self, name: &str) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.llvm()
            .append_basic_block(self.current_function().borrow().declaration().value, name)
    }

    fn set_basic_block(&self, block: inkwell::basic_block::BasicBlock<'ctx>) {
        self.builder().position_at_end(block);
    }

    fn basic_block(&self) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.builder().get_insert_block().expect("Always exists")
    }

    fn is_basic_block_terminated(&self) -> bool {
        self.basic_block()
            .get_last_instruction()
            .map(|instruction| instruction.is_terminator())
            .unwrap_or_default()
    }

    fn push_loop(
        &mut self,
        body_block: inkwell::basic_block::BasicBlock<'ctx>,
        continue_block: inkwell::basic_block::BasicBlock<'ctx>,
        join_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        self.loop_stack
            .push(Loop::new(body_block, continue_block, join_block));
    }

    fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }

    fn r#loop(&self) -> &Loop<'ctx> {
        self.loop_stack
            .last()
            .expect("The current context is not in a loop")
    }

    fn add_function(
        &mut self,
        name: &str,
        ast_id: Option<usize>,
        r#type: inkwell::types::FunctionType<'ctx>,
        return_values_length: usize,
        linkage: Option<inkwell::module::Linkage>,
    ) -> anyhow::Result<Rc<RefCell<Self::Function>>> {
        let value = self.module().add_function(name, r#type, linkage);

        let entry_block = self.llvm.append_basic_block(value, "entry");
        let return_block = self.llvm.append_basic_block(value, "return");

        let mut function = Function::new(
            name.to_owned(),
            FunctionDeclaration::new(r#type, value),
            entry_block,
            return_block,
        );
        Function::set_default_attributes(
            self.llvm,
            function.declaration(),
            self.evm_version.unwrap_or_default(),
            &self.optimizer,
        );
        function.set_debug_info(self, ast_id);
        let function = Rc::new(RefCell::new(function));
        self.functions.insert(name.to_string(), function.clone());

        self.set_current_function(name)?;
        let r#return = match return_values_length {
            0 => FunctionReturn::none(),
            1 => {
                self.set_basic_block(entry_block);
                let pointer = self.build_alloca(self.field_type(), "return_pointer")?;
                FunctionReturn::primitive(pointer)
            }
            size => {
                self.set_basic_block(entry_block);
                let pointer = self.build_alloca(
                    self.structure_type(
                        vec![self.field_type().as_basic_type_enum(); size].as_slice(),
                    ),
                    "return_pointer",
                )?;
                FunctionReturn::compound(pointer, size)
            }
        };
        function.borrow_mut().set_return(r#return);
        Ok(function)
    }

    fn get_function(&self, name: &str) -> Option<Rc<RefCell<Self::Function>>> {
        self.functions.get(name).cloned()
    }

    fn current_function(&self) -> Rc<RefCell<Self::Function>> {
        self.current_function
            .clone()
            .expect("Must be declared before use")
    }

    fn set_current_function(&mut self, name: &str) -> anyhow::Result<()> {
        let function =
            self.functions.get(name).cloned().ok_or_else(|| {
                anyhow::anyhow!("Failed to activate an undeclared function `{name}`")
            })?;
        if let Some((solidity_data, solc_debug_info_location)) = self
            .solidity_mut()
            .zip(function.borrow().solc_debug_info_location())
        {
            solidity_data.set_debug_info_solc_location(solc_debug_info_location.to_owned());
        }
        self.current_function = Some(function);
        Ok(())
    }

    fn build_call(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicValueEnum<'ctx>],
        name: &str,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
        let arguments: Vec<inkwell::values::BasicMetadataValueEnum> = arguments
            .iter()
            .copied()
            .map(inkwell::values::BasicMetadataValueEnum::from)
            .collect();
        self.build_call_metadata(function, arguments.as_slice(), name)
    }

    fn build_call_metadata(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
        let call_site_value = self.builder.build_indirect_call(
            function.r#type,
            function.value.as_global_value().as_pointer_value(),
            arguments,
            name,
        )?;

        let instruction_value = match call_site_value.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(inner) => inner.as_instruction_value(),
            inkwell::values::ValueKind::Instruction(inner) => Some(inner),
        };
        if let Some(instruction_value) = instruction_value {
            let debug_location = self.create_debug_info_location();
            instruction_value.set_debug_location(debug_location);
        }

        self.modify_call_site_value(arguments, call_site_value, function);
        Ok(call_site_value.try_as_basic_value().basic())
    }

    fn build_invoke(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicValueEnum<'ctx>],
        name: &str,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
        Self::build_call(self, function, arguments, name)
    }

    fn solidity(&self) -> Option<&Self::SolidityData> {
        self.solidity_data.as_ref()
    }

    fn solidity_mut(&mut self) -> Option<&mut Self::SolidityData> {
        self.solidity_data.as_mut()
    }

    fn set_yul_data(&mut self, data: Self::YulData) {
        self.yul_data = Some(data);
    }

    fn yul(&self) -> Option<&Self::YulData> {
        self.yul_data.as_ref()
    }

    fn yul_mut(&mut self) -> Option<&mut Self::YulData> {
        self.yul_data.as_mut()
    }

    fn set_evmla_data(&mut self, data: Self::EVMLAData) {
        self.evmla_data = Some(data);
    }

    fn evmla(&self) -> Option<&Self::EVMLAData> {
        self.evmla_data.as_ref()
    }

    fn evmla_mut(&mut self) -> Option<&mut Self::EVMLAData> {
        self.evmla_data.as_mut()
    }
}
