// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*ccode.*}}() -> !sol.string<Memory>
// CHECK:   sol.object_code "Other{{.*}}" : !sol.string<Memory>
// CHECK: sol.func @{{.*iid.*}}() -> !sol.fixedbytes<4>
// CHECK:   sol.constant 3506811462 : ui32
// CHECK: sol.func @{{.*iname.*}}() -> !sol.string<Memory>
// CHECK:   sol.string_lit "IFoo" -> !sol.string<Memory>
// CHECK: sol.func @{{.*nm.*}}() -> !sol.string<Memory>
// CHECK:   sol.string_lit "Other" -> !sol.string<Memory>
// CHECK: sol.func @{{.*rcode.*}}() -> !sol.string<Memory>
// CHECK:   sol.object_code "Other{{.*}}_deployed" : !sol.string<Memory>
// CHECK: sol.func @{{.*tmax.*}}() -> ui16
// CHECK:   sol.constant 65535 : ui16
// CHECK: sol.func @{{.*tmin.*}}() -> si8
// CHECK:   sol.constant -128 : si8

contract C {
    function ccode() public pure returns (bytes memory) { return type(Other).creationCode; }

    function iid() public pure returns (bytes4) { return type(IFoo).interfaceId; }

    function iname() public pure returns (string memory) { return type(IFoo).name; }

    function nm() public pure returns (string memory) { return type(Other).name; }

    function rcode() public pure returns (bytes memory) { return type(Other).runtimeCode; }

    function tmax() public pure returns (uint16) { return type(uint16).max; }

    function tmin() public pure returns (int8) { return type(int8).min; }
}

interface IFoo {
    function foo(uint256 x) external returns (uint256);

    function bar() external view returns (bool);
}

contract Other {
    uint256 x;
}
