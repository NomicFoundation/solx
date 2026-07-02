// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: static_call
// CHECK: try_call
// CHECK: sol.try
// CHECK: error {
// CHECK: fallback {

contract Token {
    uint256 public totalSupply;
}

contract C {
    function g(Token t) external returns (uint256) {
        try t.totalSupply() returns (uint256 s) {
            return s;
        } catch Error(string memory reason) {
            return bytes(reason).length;
        } catch {
            return 0;
        }
    }
}
