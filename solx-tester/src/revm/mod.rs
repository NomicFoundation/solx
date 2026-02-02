///
/// The REVM adapter.
///
pub mod address_iterator;
pub mod input;
pub mod revm_type_conversions;

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use colored::Colorize;
use revm::{
    ExecuteCommitEvm, InspectCommitEvm,
    context::ContextTr,
    context::Evm,
    database::{CacheState, Database, states::plain_account::PlainStorage},
    handler::{EthFrame, EthPrecompiles, instructions::EthInstructions},
    inspector::inspectors::TracerEip3155,
    interpreter::interpreter::EthInterpreter,
    primitives::{Address, FixedBytes, U256},
    state::AccountInfo,
};

use crate::revm::revm_type_conversions::web3_u256_to_revm_u256;
use crate::test::case::input::calldata::Calldata;

use self::revm_type_conversions::web3_address_to_revm_address;

/// The overloaded REVM Context type.
type Context = revm::context::Context<
    revm::context::BlockEnv,
    revm::context::TxEnv,
    revm::context::CfgEnv,
    revm::database::State<revm::database::EmptyDB>,
    revm::context::Journal<revm::database::State<revm::database::EmptyDB>>,
    (),
    revm::context::LocalContext,
>;

///
/// REVM instance with its internal state.
///
#[derive(Debug)]
pub enum REVM {
    /// REVM without tracing.
    Default(Evm<Context, (), EthInstructions<EthInterpreter, Context>, EthPrecompiles, EthFrame>),
    /// REVM with EIP-3155 tracer.
    Tracing(
        Evm<
            Context,
            TracerEip3155,
            EthInstructions<EthInterpreter, Context>,
            EthPrecompiles,
            EthFrame,
        >,
    ),
}

impl REVM {
    /// Default Ethereum chain ID.
    pub const CHAIND_ID: u64 = 1;
    /// Default gas price.
    pub const GAS_PRICE: u64 = 0;
    /// Default coinbase.
    pub const COIN_BASE: &'static str = "0x7878787878787878787878787878787878787878";
    /// Default `tx.origin`.
    pub const TX_ORIGIN: &'static str = "0x9292929292929292929292929292929292929292";

    /// Block prevrandao.
    pub const BLOCK_PREVRANDAO: &'static str =
        "0xa86c2e601b6c44eb4848f7d23d9df3113fbcac42041c49cbed5000cb4f118777";
    /// Zero block hash.
    pub const BLOCK_HASH: &'static str =
        "0x3737373737373737373737373737373737373737373737373737373737373737";
    /// Default base fee for REVM.
    pub const BASE_FEE: u64 = 0;
    /// Default block gas limit for REVM.
    pub const BLOCK_GAS_LIMIT: u64 = 30000000;
    /// Default current block number.
    pub const BLOCK_NUMBER: u128 = 2;
    /// Default current block timestamp.
    pub const BLOCK_TIMESTAMP: u128 = 30;
    /// Default timestamp step for blocks in the EVM context.
    pub const BLOCK_TIMESTAMP_STEP: u128 = 15;

    /// Gas cost for transaction fee.
    pub const GAS_COST_TRANSACTION_FEE: u64 = 21000;
    /// Gas cost for deployment fee.
    pub const GAS_COST_DEPLOYMENT_FEE: u64 = 32000;
    /// Gas cost per zero byte in calldata.
    pub const GAS_COST_PER_ZERO_CALLDATA_BYTE: u64 = 4;
    /// Gas cost per non-zero byte in calldata.
    pub const GAS_COST_PER_NON_ZERO_CALLDATA_BYTE: u64 = 16;
    /// Gas cost per byte in runtime code.
    pub const GAS_COST_PER_RUNTIME_CODE_BYTE: u64 = 200;

    ///
    /// A shortcut constructor.
    ///
    pub fn new(enable_trace: bool) -> Self {
        let mut evm = if enable_trace {
            REVM::Tracing(Evm::new_with_inspector(
                Self::context(),
                TracerEip3155::new_stdout(),
                EthInstructions::new_mainnet(),
                EthPrecompiles::default(),
            ))
        } else {
            REVM::Default(Evm::new(
                Self::context(),
                EthInstructions::new_mainnet(),
                EthPrecompiles::default(),
            ))
        };
        match &mut evm {
            REVM::Default(evm) => Self::set_data(evm),
            REVM::Tracing(evm) => Self::set_data(evm),
        }
        evm
    }

    ///
    /// Executes the transaction.
    ///
    pub fn execute_transaction(
        &mut self,
        tx: revm::context::TxEnv,
    ) -> Result<
        revm::context::result::ExecutionResult,
        revm::context::result::EVMError<
            revm::database_interface::bal::EvmDatabaseError<Infallible>,
        >,
    > {
        match self {
            REVM::Default(vm) => vm.transact_commit(tx),
            REVM::Tracing(vm) => vm.inspect_tx_commit(tx),
        }
    }

    ///
    /// Downloads the necessary compiler executables.
    ///
    pub fn download(executable_download_config_paths: Vec<PathBuf>) -> anyhow::Result<()> {
        let download_time_start = Instant::now();
        println!(
            " {} compiler executables",
            "Downloading".bright_green().bold()
        );
        for config_path in executable_download_config_paths.into_iter() {
            solx_compiler_downloader::Downloader::default().download(config_path.as_path())?;
        }
        println!(
            "    {} downloading compiler executables in {}m{:02}s",
            "Finished".bright_green().bold(),
            download_time_start.elapsed().as_secs() / 60,
            download_time_start.elapsed().as_secs() % 60,
        );

        Ok(())
    }

    ///
    /// Fills a deploy transaction with the given parameters.
    ///
    pub fn new_deploy_transaction(
        caller: web3::types::Address,
        value: Option<u128>,
        code: Vec<u8>,
    ) -> revm::context::TxEnv {
        revm::context::TxEnv::builder()
            .caller(web3_address_to_revm_address(&caller))
            .data(revm::primitives::Bytes::from(code))
            .value(revm::primitives::U256::from(value.unwrap_or_default()))
            .create()
            .gas_price(Self::GAS_PRICE as u128)
            .gas_limit(Self::BLOCK_GAS_LIMIT)
            .build_fill()
    }

    ///
    /// Fills a runtime transaction with the given parameters.
    ///
    pub fn new_runtime_transaction(
        address: web3::types::Address,
        caller: web3::types::Address,
        calldata: Calldata,
        value: Option<u128>,
    ) -> revm::context::TxEnv {
        revm::context::TxEnv::builder()
            .caller(web3_address_to_revm_address(&caller))
            .data(revm::primitives::Bytes::from(calldata.inner))
            .value(revm::primitives::U256::from(value.unwrap_or_default()))
            .to(web3_address_to_revm_address(&address))
            .gas_price(Self::GAS_PRICE as u128)
            .gas_limit(Self::BLOCK_GAS_LIMIT)
            .build_fill()
    }

    ///
    /// Sets the account data and balance.
    ///
    pub fn set_account(&mut self, account: &web3::types::Address, balance: web3::types::U256) {
        let address = web3_address_to_revm_address(account);
        let balance = web3_u256_to_revm_u256(balance);

        let nonce = self
            .db_mut()
            .basic(address)
            .map(|info| info.map(|info| info.nonce).unwrap_or(1))
            .unwrap_or(1);
        let account_info = revm::state::AccountInfo {
            balance,
            code_hash: revm::primitives::KECCAK_EMPTY,
            code: None,
            nonce,
            account_id: None,
        };

        self.db_mut().insert_account(address, account_info);
    }

    ///
    /// Returns addresses that must be funded for testing.
    ///
    pub fn get_rich_addresses() -> Vec<web3::types::Address> {
        (0..=9)
            .map(|address_id| {
                format!(
                    "0x121212121212121212121212121212000000{}{}",
                    address_id, "012"
                )
            })
            .map(|string| web3::types::Address::from_str(&string).unwrap())
            .collect()
    }

    ///
    /// Sets the account storage.
    ///
    pub fn extend_account_storage(
        &mut self,
        account: &web3::types::Address,
        storage: HashMap<web3::types::U256, web3::types::U256>,
    ) {
        let address = web3_address_to_revm_address(account);
        let storage: HashMap<revm::primitives::U256, revm::primitives::U256> = storage
            .into_iter()
            .map(|(key, value)| (web3_u256_to_revm_u256(key), web3_u256_to_revm_u256(value)))
            .collect();

        let account_info = self
            .db_mut()
            .basic(address)
            .expect("Always exists")
            .unwrap_or_default();
        let mut existing_storage = self
            .db_mut()
            .cache
            .accounts
            .get(&address)
            .and_then(|account| account.account.as_ref())
            .map(|account| account.storage.to_owned())
            .unwrap_or_default();
        existing_storage.extend(storage);

        self.db_mut()
            .insert_account_with_storage(address, account_info, existing_storage);
    }

    ///
    /// Sets the block data.
    ///
    pub fn set_block_data(
        &mut self,
        number: revm::primitives::U256,
        timestamp: revm::primitives::U256,
    ) {
        let block = match self {
            REVM::Default(evm) => &mut evm.block,
            REVM::Tracing(evm) => &mut evm.block,
        };
        block.number = number;
        block.timestamp = timestamp;
    }

    ///
    /// Returns the calldata gas cost.
    ///
    pub fn calldata_gas_cost(calldata: &[u8]) -> u64 {
        calldata
            .iter()
            .map(|byte| {
                if *byte == 0 {
                    Self::GAS_COST_PER_ZERO_CALLDATA_BYTE
                } else {
                    Self::GAS_COST_PER_NON_ZERO_CALLDATA_BYTE
                }
            })
            .sum::<u64>()
    }

    ///
    /// With the total gas usage, calldata, and init code, returns the gas used for deploy bytecode execution only.
    ///
    pub fn deploy_bytecode_execution_gas(
        total_gas_used: u64,
        calldata_cost: u64,
        deploy_code_size: usize,
        runtime_code_size: usize,
    ) -> u64 {
        let init_code_fee = 2
            * (((deploy_code_size as f64) / (solx_utils::BYTE_LENGTH_FIELD as f64)).ceil() as u64);
        let runtime_code_fee = (runtime_code_size as u64) * Self::GAS_COST_PER_RUNTIME_CODE_BYTE;
        total_gas_used.saturating_sub(
            Self::GAS_COST_TRANSACTION_FEE
                + calldata_cost
                + Self::GAS_COST_DEPLOYMENT_FEE
                + init_code_fee
                + runtime_code_fee,
        )
    }

    ///
    /// With the total gas usage and calldata, returns the gas used for runtime bytecode execution only.
    ///
    pub fn runtime_bytecode_execution_gas(total_gas_used: u64, calldata_cost: u64) -> u64 {
        total_gas_used.saturating_sub(Self::GAS_COST_TRANSACTION_FEE + calldata_cost)
    }

    ///
    /// Immutable reference to the database.
    ///
    pub fn db(&self) -> &revm::database::State<revm::database::EmptyDB> {
        match self {
            REVM::Default(evm) => evm.db(),
            REVM::Tracing(evm) => evm.db(),
        }
    }

    ///
    /// Mutable reference to the database.
    ///
    pub fn db_mut(&mut self) -> &mut revm::database::State<revm::database::EmptyDB> {
        match self {
            REVM::Default(evm) => evm.db_mut(),
            REVM::Tracing(evm) => evm.db_mut(),
        }
    }

    ///
    /// Immutable reference to the context database.
    ///
    pub fn ctx_db(&self) -> &revm::database::State<revm::database::EmptyDB> {
        match self {
            REVM::Default(evm) => evm.ctx.db(),
            REVM::Tracing(evm) => evm.ctx.db(),
        }
    }

    ///
    /// Builds the default context for REVM.
    ///
    fn context() -> Context {
        let mut cache = CacheState::new(false);
        // Account 0x00 needs to have its code hash on 0.
        cache.insert_account_with_storage(
            Address::from_word(FixedBytes::from(U256::ZERO)),
            AccountInfo {
                balance: U256::from(0_u64),
                code_hash: FixedBytes::from(U256::ZERO),
                code: None,
                nonce: 1,
                account_id: None,
            },
            PlainStorage::default(),
        );
        // Precompile 0x01 needs to have its code hash.
        cache.insert_account_with_storage(
            Address::from_word(FixedBytes::from(U256::from(1_u64))),
            AccountInfo {
                balance: U256::from(1_u64),
                code_hash: FixedBytes::from_str(
                    "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
                )
                .expect("Always valid"),
                code: None,
                nonce: 1,
                account_id: None,
            },
            PlainStorage::default(),
        );

        let block_hashes = (0..0xffff - 0x3737)
            .enumerate()
            .map(|(index, value)| {
                let hash = format!(
                    "0x373737373737373737373737373737373737373737373737373737373737{:04x}",
                    0x3737 + value
                );
                (
                    index as u64,
                    revm::primitives::B256::from_str(hash.as_str()).expect("Always valid"),
                )
            })
            .collect();

        let state = revm::database::State::builder()
            .with_cached_prestate(cache)
            .with_block_hashes(block_hashes)
            .with_bundle_update()
            .build();

        Context::new(state, revm::primitives::hardfork::PRAGUE)
    }

    ///
    /// Sets the initial REVM data.
    ///
    fn set_data<INSP>(
        evm: &mut Evm<
            Context,
            INSP,
            EthInstructions<EthInterpreter, Context>,
            EthPrecompiles,
            EthFrame,
        >,
    ) {
        evm.block.beneficiary =
            revm::primitives::Address::from_str(Self::COIN_BASE).expect("Always valid");
        evm.block.basefee = Self::BASE_FEE;
        evm.block.difficulty =
            revm::primitives::U256::from_str(Self::BLOCK_PREVRANDAO).expect("Always valid");
        evm.block.prevrandao =
            Some(revm::primitives::B256::from_str(Self::BLOCK_PREVRANDAO).expect("Always valid"));
        evm.block.gas_limit = Self::BLOCK_GAS_LIMIT;
        evm.block.timestamp = revm::primitives::U256::from(Self::BLOCK_TIMESTAMP);
        evm.tx.chain_id = Some(Self::CHAIND_ID);
        evm.cfg.disable_nonce_check = true;
    }
}
