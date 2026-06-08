// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A `try`/`catch` over an external call lowers to a single `sol.try` carrying
// the success flag and four regions. Each present clause populates its region;
// the typed `Error(string)` / `Panic(uint256)` payloads and the parameter-less
// fallback are delivered structurally, and the op's lowering owns the selector
// dispatch and decode. solx and solc emit the same `sol.try` shape.

// CHECK: try_call
// CHECK: sol.try
// CHECK: panic {
// CHECK: ^bb0({{.*}}: ui256):
// CHECK: error {
// CHECK: ^bb0({{.*}}: !sol.string<Memory>):
// CHECK: fallback {

interface I {
    function f() external returns (uint256);
}

contract C {
    function g(I i) external returns (uint256) {
        try i.f() returns (uint256 v) {
            return v;
        } catch Error(string memory reason) {
            return bytes(reason).length;
        } catch Panic(uint256 code) {
            return code;
        } catch {
            return 7;
        }
    }
}
