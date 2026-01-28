//!
//! Unit test constants.
//!

/// A test input file.
pub const TEST_CONTRACTS_PATH: &str = "tests/data/contracts/";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_PATH: &str = "tests/data/contracts/solidity/Test.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_CALLER_MAIN_PATH: &str =
    "tests/data/contracts/solidity/caller/Main.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_CALLER_CALLABLE_PATH: &str =
    "tests/data/contracts/solidity/caller/Callable.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_SIMPLE_CONTRACT_PATH: &str =
    "tests/data/contracts/solidity/SimpleContract.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_LINKER_MIXED_DEPS_PATH: &str =
    "tests/data/contracts/solidity/LinkedMixedDeps.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_LINKER_MIXED_DEPS_MULTI_LEVEL_PATH: &str =
    "tests/data/contracts/solidity/LinkedMixedDepsMultiLevel.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_OPTIMIZED_PATH: &str =
    "tests/data/contracts/solidity/Optimized.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_STACK_TOO_DEEP_SOLC_PATH: &str =
    "tests/data/contracts/solidity/StackTooDeepSolc.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_STACK_TOO_DEEP_LLVM_PATH: &str =
    "tests/data/contracts/solidity/StackTooDeepLLVM.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_INTERFACE_EMPTY_YUL_PATH: &str =
    "tests/data/contracts/solidity/InterfaceEmptyYul.sol";

/// A test input file.
pub const TEST_SOLIDITY_CONTRACT_FUZZED_LINKER_ERROR_PATH: &str =
    "tests/data/contracts/solidity/FuzzedLinkerError.sol";

/// A test input file.
pub const TEST_YUL_CONTRACT_PATH: &str = "tests/data/contracts/yul/Test.yul";

/// A test input file.
pub const TEST_YUL_CONTRACT_OBJECT_NAMING_PATH: &str = "tests/data/contracts/yul/ObjectNaming.yul";

/// A test input file.
pub const TEST_LLVM_IR_CONTRACT_PATH: &str = "tests/data/contracts/llvm_ir/Test.ll";

/// A test input file.
pub const TEST_LLVM_IR_CONTRACT_INVALID_PATH: &str = "tests/data/contracts/llvm_ir/Invalid.ll";

/// A test input file.
pub const TEST_LLVM_IR_CONTRACT_LINKER_ERROR_PATH: &str =
    "tests/data/contracts/llvm_ir/LinkerError.ll";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_NON_EXISTENT_PATH: &str =
    "tests/data/standard_json_input/non_existent.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_INVALID_UTF8_PATH: &str =
    "tests/data/standard_json_input/invalid_utf8.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_PATH: &str = "tests/data/standard_json_input/solidity.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_DEPLOY_TIME_LINKING_PATH: &str =
    "tests/data/standard_json_input/solidity_deploy_time_linking.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_FUZZED_SIMPLE_USE_EXPRESSION_PATH: &str =
    "tests/data/standard_json_input/solidity_fuzzed_simple_use_expression.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_EMPTY_SOURCES_PATH: &str =
    "tests/data/standard_json_input/solidity_empty_sources.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_MISSING_SOURCES_PATH: &str =
    "tests/data/standard_json_input/solidity_missing_sources.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_INVALID_PATH: &str =
    "tests/data/standard_json_input/solidity_invalid.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_RECURSION_PATH: &str =
    "tests/data/standard_json_input/solidity_recursion.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_PATH: &str =
    "tests/data/standard_json_input/select_evm.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_BYTECODE_PATH: &str =
    "tests/data/standard_json_input/select_evm_bytecode.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_BYTECODE_OPCODES_PATH: &str =
    "tests/data/standard_json_input/select_evm_bytecode_opcodes.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_DEPLOYED_BYTECODE_PATH: &str =
    "tests/data/standard_json_input/select_evm_deployed_bytecode.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_DEPLOYED_BYTECODE_LINK_REFERENCES_PATH: &str =
    "tests/data/standard_json_input/select_evm_deployed_bytecode_link_references.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_SELECT_SINGLE_PATH: &str =
    "tests/data/standard_json_input/select_single.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_SELECT_NONE_PATH: &str =
    "tests/data/standard_json_input/select_none.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_SELECT_ALL_PATH: &str =
    "tests/data/standard_json_input/select_all.json";

/// A test input file.
pub const TEST_SOLIDITY_STANDARD_JSON_SELECT_ALL_WILDCARD_PATH: &str =
    "tests/data/standard_json_input/select_all_wildcard.json";

/// A test input file.
pub const TEST_YUL_STANDARD_JSON_PATH: &str = "tests/data/standard_json_input/yul.json";

/// A test input file.
pub const TEST_YUL_STANDARD_JSON_URLS_PATH: &str = "tests/data/standard_json_input/yul_urls.json";

/// A test input file.
pub const TEST_YUL_STANDARD_JSON_URLS_INVALID_PATH: &str =
    "tests/data/standard_json_input/yul_urls_invalid.json";

/// A test input file.
pub const TEST_YUL_STANDARD_JSON_URLS_DEBUG_INFO_PATH: &str =
    "tests/data/standard_json_input/yul_urls_debug_info.json";

/// A test input file.
pub const TEST_LLVM_IR_STANDARD_JSON_PATH: &str =
    "tests/data/standard_json_input/llvm_ir_urls.json";

/// A test input file.
pub const TEST_LLVM_IR_STANDARD_JSON_INVALID_PATH: &str =
    "tests/data/standard_json_input/llvm_ir_urls_invalid.json";

/// A test input file.
pub const TEST_LLVM_IR_STANDARD_JSON_MISSING_FILE_PATH: &str =
    "tests/data/standard_json_input/llvm_ir_urls_missing_file.json";

/// A test input file.
pub const TEST_LLVM_IR_STANDARD_JSON_DEBUG_INFO_PATH: &str =
    "tests/data/standard_json_input/llvm_ir_urls_debug_info.json";

/// A test input file.
pub const TEST_JSON_METADATA_HASH_IPFS_AND_METADATA: &str =
    "tests/data/standard_json_input/metadata_hash_ipfs_and_metadata.json";

/// A test input file.
pub const TEST_JSON_METADATA_HASH_IPFS_NO_METADATA: &str =
    "tests/data/standard_json_input/metadata_hash_ipfs_no_metadata.json";

/// A test input file.
pub const TEST_JSON_METADATA_HASH_NONE_AND_METADATA: &str =
    "tests/data/standard_json_input/metadata_hash_none_and_metadata.json";

/// A test input file.
pub const TEST_JSON_METADATA_HASH_NONE_NO_METADATA: &str =
    "tests/data/standard_json_input/metadata_hash_none_no_metadata.json";

/// A test input file.
pub const TEST_JSON_NO_CBOR_METADATA: &str = "tests/data/standard_json_input/no_cbor_metadata.json";

/// A test input file.
pub const TEST_JSON_EVM_VERSION_TOO_OLD: &str =
    "tests/data/standard_json_input/evm_version_too_old.json";

/// The broken input file path.
pub const TEST_BROKEN_INPUT_PATH: &str = "tests/data/broken.bad";

/// A non-existent path.
pub const TEST_NON_EXISTENT_PATH: &str = "tests/data/non_existent";

/// A test constant.
pub const LIBRARY_DEFAULT: &str = "tests/data/contracts/solidity/MiniMath.sol:MiniMath=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC";

/// A test constant.
pub const LIBRARY_CONTRACT_NAME_MISSING: &str =
    "tests/data/contracts/solidity/MiniMath.sol=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC";

/// A test constant.
pub const LIBRARY_ADDRESS_MISSING: &str = "tests/data/contracts/solidity/MiniMath.sol:MiniMath";

/// A test constant.
pub const LIBRARY_ADDRESS_INVALID: &str =
    "tests/data/contracts/solidity/MiniMath.sol:MiniMath=INVALID";
