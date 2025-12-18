//!
//! The LLVM intrinsic functions.
//!

use inkwell::types::BasicType;

use crate::codegen::context::address_space::AddressSpace;
use crate::context::function::declaration::Declaration as FunctionDeclaration;

///
/// The LLVM intrinsic functions, implemented in the LLVM back-end.
///
/// Most of them are translated directly into bytecode instructions.
///
#[derive(Debug)]
pub struct Intrinsics<'ctx> {
    /// The corresponding intrinsic function name.
    pub ctlz: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub exp: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub signextend: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub sha3: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub addmod: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub mulmod: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub byte: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub mstore8: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub msize: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub calldatasize: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub returndatasize: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub codesize: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub extcodesize: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub extcodecopy: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub extcodehash: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub datasize: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub dataoffset: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub linkersymbol: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub loadimmutable: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub pushdeployaddress: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub log0: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub log1: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub log2: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub log3: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub log4: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub call: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub staticcall: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub delegatecall: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub callcode: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub create: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub create2: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub address: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub caller: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub balance: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub selfbalance: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub callvalue: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub gas: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub gasprice: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub gaslimit: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub blockhash: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub coinbase: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub basefee: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub timestamp: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub number: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub chainid: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub origin: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub difficulty: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub r#return: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub revert: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub stop: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub invalid: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub selfdestruct: FunctionDeclaration<'ctx>,

    /// The corresponding intrinsic function name.
    pub memory_move_heap: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub memory_copy_from_calldata: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub memory_copy_from_return_data: FunctionDeclaration<'ctx>,
    /// The corresponding intrinsic function name.
    pub memory_copy_from_code: FunctionDeclaration<'ctx>,
}

impl<'ctx> Intrinsics<'ctx> {
    /// The corresponding intrinsic function name.
    pub const FUNCTION_CTLZ: &'static str = "llvm.ctlz.i256";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_EXP: &'static str = "llvm.evm.exp";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_SIGNEXTEND: &'static str = "llvm.evm.signextend";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_SHA3: &'static str = "llvm.evm.sha3";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_ADDMOD: &'static str = "llvm.evm.addmod";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_MULMOD: &'static str = "llvm.evm.mulmod";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_BYTE: &'static str = "llvm.evm.byte";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_MSTORE8: &'static str = "llvm.evm.mstore8";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_MSIZE: &'static str = "llvm.evm.msize";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CALLDATASIZE: &'static str = "llvm.evm.calldatasize";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_RETURNDATASIZE: &'static str = "llvm.evm.returndatasize";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CODESIZE: &'static str = "llvm.evm.codesize";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_EXTCODESIZE: &'static str = "llvm.evm.extcodesize";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_EXTCODECOPY: &'static str = "llvm.evm.extcodecopy";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_EXTCODEHASH: &'static str = "llvm.evm.extcodehash";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_DATASIZE: &'static str = "llvm.evm.datasize";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_DATAOFFSET: &'static str = "llvm.evm.dataoffset";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_LINKER_SYMBOL: &'static str = "llvm.evm.linkersymbol";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_LOAD_IMMUTABLE: &'static str = "llvm.evm.loadimmutable";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_PUSH_DEPLOY_ADDRESS: &'static str = "llvm.evm.pushdeployaddress";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_LOG0: &'static str = "llvm.evm.log0";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_LOG1: &'static str = "llvm.evm.log1";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_LOG2: &'static str = "llvm.evm.log2";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_LOG3: &'static str = "llvm.evm.log3";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_LOG4: &'static str = "llvm.evm.log4";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CALL: &'static str = "llvm.evm.call";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_STATICCALL: &'static str = "llvm.evm.staticcall";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_DELEGATECALL: &'static str = "llvm.evm.delegatecall";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CODECALL: &'static str = "llvm.evm.callcode";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CREATE: &'static str = "llvm.evm.create";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CREATE2: &'static str = "llvm.evm.create2";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_ADDRESS: &'static str = "llvm.evm.address";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CALLER: &'static str = "llvm.evm.caller";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_BALANCE: &'static str = "llvm.evm.balance";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_SELFBALANCE: &'static str = "llvm.evm.selfbalance";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CALLVALUE: &'static str = "llvm.evm.callvalue";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_GAS: &'static str = "llvm.evm.gas";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_GASPRICE: &'static str = "llvm.evm.gasprice";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_GASLIMIT: &'static str = "llvm.evm.gaslimit";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_BLOCKHASH: &'static str = "llvm.evm.blockhash";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_COINBASE: &'static str = "llvm.evm.coinbase";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_BASEFEE: &'static str = "llvm.evm.basefee";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_TIMESTAMP: &'static str = "llvm.evm.timestamp";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_NUMBER: &'static str = "llvm.evm.number";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CHAINID: &'static str = "llvm.evm.chainid";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_ORIGIN: &'static str = "llvm.evm.origin";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_DIFFICULTY: &'static str = "llvm.evm.difficulty";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_RETURN: &'static str = "llvm.evm.return";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_REVERT: &'static str = "llvm.evm.revert";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_STOP: &'static str = "llvm.evm.stop";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_INVALID: &'static str = "llvm.evm.invalid";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_SELFDESTRUCT: &'static str = "llvm.evm.selfdestruct";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_MEMORY_MOVE_HEAP: &'static str = "llvm.memmove.p1.p1.i256";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_MEMORY_COPY_FROM_CALLDATA: &'static str = "llvm.memcpy.p1.p2.i256";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_MEMORY_COPY_FROM_RETURN_DATA: &'static str = "llvm.memcpy.p1.p3.i256";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_MEMORY_COPY_FROM_CODE: &'static str = "llvm.memcpy.p1.p4.i256";

    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        llvm: &'ctx inkwell::context::Context,
        module: &inkwell::module::Module<'ctx>,
    ) -> Self {
        let void_type = llvm.void_type();
        let bool_type = llvm.bool_type();
        let field_type = llvm.custom_width_int_type(solx_utils::BIT_LENGTH_FIELD as u32);

        let heap_byte_pointer_type = llvm.ptr_type(AddressSpace::Heap.into());
        let calldata_byte_pointer_type = llvm.ptr_type(AddressSpace::Calldata.into());
        let return_data_byte_pointer_type = llvm.ptr_type(AddressSpace::ReturnData.into());
        let code_byte_pointer_type = llvm.ptr_type(AddressSpace::Code.into());

        let ctlz = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CTLZ,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    bool_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );

        let exp = Self::declare(
            llvm,
            module,
            Self::FUNCTION_EXP,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let signextend = Self::declare(
            llvm,
            module,
            Self::FUNCTION_SIGNEXTEND,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let sha3 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_SHA3,
            field_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let addmod = Self::declare(
            llvm,
            module,
            Self::FUNCTION_ADDMOD,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let mulmod = Self::declare(
            llvm,
            module,
            Self::FUNCTION_MULMOD,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let byte = Self::declare(
            llvm,
            module,
            Self::FUNCTION_BYTE,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );

        let mstore8 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_MSTORE8,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let msize = Self::declare(
            llvm,
            module,
            Self::FUNCTION_MSIZE,
            field_type.fn_type(&[], false),
        );
        let calldatasize = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CALLDATASIZE,
            field_type.fn_type(&[], false),
        );
        let returndatasize = Self::declare(
            llvm,
            module,
            Self::FUNCTION_RETURNDATASIZE,
            field_type.fn_type(&[], false),
        );
        let codesize = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CODESIZE,
            field_type.fn_type(&[], false),
        );
        let extcodesize = Self::declare(
            llvm,
            module,
            Self::FUNCTION_EXTCODESIZE,
            field_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );
        let extcodecopy = Self::declare(
            llvm,
            module,
            Self::FUNCTION_EXTCODECOPY,
            void_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    code_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let extcodehash = Self::declare(
            llvm,
            module,
            Self::FUNCTION_EXTCODEHASH,
            field_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );
        let datasize = Self::declare(
            llvm,
            module,
            Self::FUNCTION_DATASIZE,
            field_type.fn_type(&[llvm.metadata_type().into()], false),
        );
        let dataoffset = Self::declare(
            llvm,
            module,
            Self::FUNCTION_DATAOFFSET,
            field_type.fn_type(&[llvm.metadata_type().into()], false),
        );
        let linkersymbol = Self::declare(
            llvm,
            module,
            Self::FUNCTION_LINKER_SYMBOL,
            field_type.fn_type(&[llvm.metadata_type().into()], false),
        );
        let loadimmutable = Self::declare(
            llvm,
            module,
            Self::FUNCTION_LOAD_IMMUTABLE,
            field_type.fn_type(&[llvm.metadata_type().into()], false),
        );
        let pushdeployaddress = Self::declare(
            llvm,
            module,
            Self::FUNCTION_PUSH_DEPLOY_ADDRESS,
            field_type.fn_type(&[], false),
        );

        let log0 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_LOG0,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let log1 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_LOG1,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let log2 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_LOG2,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let log3 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_LOG3,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let log4 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_LOG4,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let call = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CALL,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let staticcall = Self::declare(
            llvm,
            module,
            Self::FUNCTION_STATICCALL,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let delegatecall = Self::declare(
            llvm,
            module,
            Self::FUNCTION_DELEGATECALL,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let callcode = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CODECALL,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let create = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CREATE,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let create2 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CREATE2,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let address = Self::declare(
            llvm,
            module,
            Self::FUNCTION_ADDRESS,
            field_type.fn_type(&[], false),
        );
        let caller = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CALLER,
            field_type.fn_type(&[], false),
        );
        let balance = Self::declare(
            llvm,
            module,
            Self::FUNCTION_BALANCE,
            field_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );
        let selfbalance = Self::declare(
            llvm,
            module,
            Self::FUNCTION_SELFBALANCE,
            field_type.fn_type(&[], false),
        );
        let callvalue = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CALLVALUE,
            field_type.fn_type(&[], false),
        );
        let gas = Self::declare(
            llvm,
            module,
            Self::FUNCTION_GAS,
            field_type.fn_type(&[], false),
        );
        let gasprice = Self::declare(
            llvm,
            module,
            Self::FUNCTION_GASPRICE,
            field_type.fn_type(&[], false),
        );
        let gaslimit = Self::declare(
            llvm,
            module,
            Self::FUNCTION_GASLIMIT,
            field_type.fn_type(&[], false),
        );
        let blockhash = Self::declare(
            llvm,
            module,
            Self::FUNCTION_BLOCKHASH,
            field_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );
        let coinbase = Self::declare(
            llvm,
            module,
            Self::FUNCTION_COINBASE,
            field_type.fn_type(&[], false),
        );
        let basefee = Self::declare(
            llvm,
            module,
            Self::FUNCTION_BASEFEE,
            field_type.fn_type(&[], false),
        );
        let timestamp = Self::declare(
            llvm,
            module,
            Self::FUNCTION_TIMESTAMP,
            field_type.fn_type(&[], false),
        );
        let number = Self::declare(
            llvm,
            module,
            Self::FUNCTION_NUMBER,
            field_type.fn_type(&[], false),
        );
        let chainid = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CHAINID,
            field_type.fn_type(&[], false),
        );
        let origin = Self::declare(
            llvm,
            module,
            Self::FUNCTION_ORIGIN,
            field_type.fn_type(&[], false),
        );
        let difficulty = Self::declare(
            llvm,
            module,
            Self::FUNCTION_DIFFICULTY,
            field_type.fn_type(&[], false),
        );

        let r#return = Self::declare(
            llvm,
            module,
            Self::FUNCTION_RETURN,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let revert = Self::declare(
            llvm,
            module,
            Self::FUNCTION_REVERT,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let stop = Self::declare(
            llvm,
            module,
            Self::FUNCTION_STOP,
            void_type.fn_type(&[], false),
        );
        let invalid = Self::declare(
            llvm,
            module,
            Self::FUNCTION_INVALID,
            void_type.fn_type(&[], false),
        );

        let selfdestruct = Self::declare(
            llvm,
            module,
            Self::FUNCTION_SELFDESTRUCT,
            void_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );

        let memory_move_heap = Self::declare(
            llvm,
            module,
            Self::FUNCTION_MEMORY_MOVE_HEAP,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    bool_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let memory_copy_from_calldata = Self::declare(
            llvm,
            module,
            Self::FUNCTION_MEMORY_COPY_FROM_CALLDATA,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    calldata_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    bool_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let memory_copy_from_return_data = Self::declare(
            llvm,
            module,
            Self::FUNCTION_MEMORY_COPY_FROM_RETURN_DATA,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    return_data_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    bool_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let memory_copy_from_code = Self::declare(
            llvm,
            module,
            Self::FUNCTION_MEMORY_COPY_FROM_CODE,
            void_type.fn_type(
                &[
                    heap_byte_pointer_type.as_basic_type_enum().into(),
                    code_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    bool_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );

        Self {
            ctlz,

            exp,
            signextend,
            sha3,
            addmod,
            mulmod,
            byte,

            mstore8,
            msize,
            calldatasize,
            returndatasize,
            codesize,
            extcodesize,
            extcodecopy,
            extcodehash,
            datasize,
            dataoffset,
            linkersymbol,
            loadimmutable,
            pushdeployaddress,

            log0,
            log1,
            log2,
            log3,
            log4,

            call,
            staticcall,
            delegatecall,
            callcode,

            create,
            create2,

            address,
            caller,
            balance,
            selfbalance,
            callvalue,
            gas,
            gasprice,
            gaslimit,
            blockhash,
            coinbase,
            basefee,
            timestamp,
            number,
            chainid,
            origin,
            difficulty,

            r#return,
            revert,
            stop,
            invalid,

            selfdestruct,

            memory_move_heap,
            memory_copy_from_calldata,
            memory_copy_from_return_data,
            memory_copy_from_code,
        }
    }

    ///
    /// Finds the specified LLVM intrinsic function in the target and returns its declaration.
    ///
    pub fn declare(
        llvm: &'ctx inkwell::context::Context,
        module: &inkwell::module::Module<'ctx>,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
    ) -> FunctionDeclaration<'ctx> {
        let intrinsic = inkwell::intrinsics::Intrinsic::find(name)
            .unwrap_or_else(|| panic!("Intrinsic function `{name}` does not exist"));
        let argument_types = Self::argument_types(llvm, name);
        let value = intrinsic
            .get_declaration(module, argument_types.as_slice())
            .unwrap_or_else(|| panic!("Intrinsic function `{name}` declaration error"));
        FunctionDeclaration::new(r#type, value)
    }

    ///
    /// Returns the LLVM types for selecting via the signature.
    ///
    pub fn argument_types(
        llvm: &'ctx inkwell::context::Context,
        name: &str,
    ) -> Vec<inkwell::types::BasicTypeEnum<'ctx>> {
        let field_type = llvm.custom_width_int_type(solx_utils::BIT_LENGTH_FIELD as u32);

        match name {
            name if name == Self::FUNCTION_MEMORY_MOVE_HEAP => vec![
                llvm.ptr_type(AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                llvm.ptr_type(AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                field_type.as_basic_type_enum(),
            ],
            name if name == Self::FUNCTION_MEMORY_COPY_FROM_CALLDATA => vec![
                llvm.ptr_type(AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                llvm.ptr_type(AddressSpace::Calldata.into())
                    .as_basic_type_enum(),
                field_type.as_basic_type_enum(),
            ],
            name if name == Self::FUNCTION_MEMORY_COPY_FROM_RETURN_DATA => vec![
                llvm.ptr_type(AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                llvm.ptr_type(AddressSpace::ReturnData.into())
                    .as_basic_type_enum(),
                field_type.as_basic_type_enum(),
            ],
            name if name == Self::FUNCTION_MEMORY_COPY_FROM_CODE => vec![
                llvm.ptr_type(AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                llvm.ptr_type(AddressSpace::Code.into())
                    .as_basic_type_enum(),
                field_type.as_basic_type_enum(),
            ],
            _ => vec![],
        }
    }
}
