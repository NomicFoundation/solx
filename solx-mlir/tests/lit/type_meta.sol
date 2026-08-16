// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*contract_creation_code.*}}() -> !sol.string<Memory>
// CHECK:   sol.object_code "{{[^"]*}}Other{{[0-9_]*}}" : !sol.string<Memory>

// CHECK: sol.func @{{.*contract_name.*}}() -> !sol.string<Memory>
// CHECK:   sol.string_lit "Other" -> !sol.string<Memory>

// CHECK: sol.func @{{.*contract_runtime_code.*}}() -> !sol.string<Memory>
// CHECK:   sol.object_code "{{[^"]*}}Other{{[0-9_]*}}_deployed" : !sol.string<Memory>

// CHECK: sol.func @{{.*integer_max.*}}() -> ui16
// CHECK:   sol.constant 65535 : ui16

// CHECK: sol.func @{{.*integer_min.*}}() -> si8
// CHECK:   sol.constant -128 : si8

// CHECK: sol.func @{{.*interface_id.*}}() -> !sol.fixedbytes<4>
// CHECK:   sol.constant 3506811462 : ui32

// CHECK: sol.func @{{.*interface_name.*}}() -> !sol.string<Memory>
// CHECK:   sol.string_lit "IFoo" -> !sol.string<Memory>

// CHECK: sol.func @{{.*library_creation_code.*}}() -> !sol.string<Memory>
// CHECK:   sol.object_code "{{[^"]*}}Registry{{[0-9_]*}}" : !sol.string<Memory>

// CHECK: sol.func @{{.*library_name.*}}() -> !sol.string<Memory>
// CHECK:   sol.string_lit "Registry" -> !sol.string<Memory>

// CHECK: sol.func @{{.*library_runtime_code.*}}() -> !sol.string<Memory>
// CHECK:   sol.object_code "{{[^"]*}}Registry{{[0-9_]*}}_deployed" : !sol.string<Memory>

// CHECK: sol.func @{{.*interface_id_over_non_functions.*}}() -> !sol.fixedbytes<4>
// CHECK:   sol.constant 1547088262 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.return

interface IFoo {
    function foo(uint256 x) external returns (uint256);

    function bar() external view returns (bool);
}

contract Other {
    uint256 x;
}

contract Meta {
    function contract_creation_code() public pure returns (bytes memory) {
        return type(Other).creationCode;
    }

    function contract_name() public pure returns (string memory) {
        return type(Other).name;
    }

    function contract_runtime_code() public pure returns (bytes memory) {
        return type(Other).runtimeCode;
    }

    function integer_max() public pure returns (uint16) {
        return type(uint16).max;
    }

    function integer_min() public pure returns (int8) {
        return type(int8).min;
    }

    function interface_id() public pure returns (bytes4) {
        return type(IFoo).interfaceId;
    }

    function interface_name() public pure returns (string memory) {
        return type(IFoo).name;
    }

    function library_creation_code() public pure returns (bytes memory) {
        return type(Registry).creationCode;
    }

    function library_name() public pure returns (string memory) {
        return type(Registry).name;
    }

    function library_runtime_code() public pure returns (bytes memory) {
        return type(Registry).runtimeCode;
    }
}

interface I {
    event Ping(uint256 x);

    error Bad(uint256 y);

    function ping() external returns (uint256);
}

contract NonFunctionMembers {
    function interface_id_over_non_functions() public pure returns (bytes4) {
        return type(I).interfaceId;
    }
}

library Registry {}
