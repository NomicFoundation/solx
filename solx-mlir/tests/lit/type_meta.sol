// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// type(C).name -> a `string memory` literal; type(I).interfaceId -> the XOR of the
// interface's function selectors as a ui32 constant bridged to bytes4; type(C).
// creationCode/runtimeCode -> sol.object_code over the contract / its `_deployed`
// object. solx walks functions alphabetically, solc in source order; the object
// symbol carries a solc node-id suffix, so it is matched with a regex. CHECK-DAG.

// CHECK-DAG: sol.func @{{.*nm.*}}() -> !sol.string<Memory>
// CHECK-DAG:   sol.string_lit "Other" -> !sol.string<Memory>
// CHECK-DAG: sol.func @{{.*iname.*}}() -> !sol.string<Memory>
// CHECK-DAG:   sol.string_lit "IFoo" -> !sol.string<Memory>
// CHECK-DAG: sol.func @{{.*iid.*}}() -> !sol.fixedbytes<4>
// CHECK-DAG:   sol.constant 3506811462 : ui32
// CHECK-DAG: sol.func @{{.*ccode.*}}() -> !sol.string<Memory>
// CHECK-DAG:   sol.object_code "Other{{.*}}" : !sol.string<Memory>
// CHECK-DAG: sol.func @{{.*rcode.*}}() -> !sol.string<Memory>
// CHECK-DAG:   sol.object_code "Other{{.*}}_deployed" : !sol.string<Memory>

interface IFoo {
    function foo(uint256 x) external returns (uint256);
    function bar() external view returns (bool);
}
contract Other {
    uint256 x;
}
contract C {
    function nm() public pure returns (string memory) { return type(Other).name; }
    function iname() public pure returns (string memory) { return type(IFoo).name; }
    function iid() public pure returns (bytes4) { return type(IFoo).interfaceId; }
    function ccode() public pure returns (bytes memory) { return type(Other).creationCode; }
    function rcode() public pure returns (bytes memory) { return type(Other).runtimeCode; }
}
