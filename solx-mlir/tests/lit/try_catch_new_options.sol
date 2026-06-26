// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}make{{.*}}(%arg0: ui256) -> ui256
// CHECK: sol.new "Created{{.*}}" value = %{{.*}} salt = %{{.*}} ctor(%{{.*}} : ui256) try : !sol.contract<"Created{{.*}}">
// CHECK: %[[ST:.*]] = sol.cmp ne, %{{.*}}, %{{.*}} : ui160
// CHECK: sol.try %[[ST]] {
// CHECK: } fallback {

contract Created {
    uint256 public x;

    constructor(uint256 v) payable {
        x = v;
    }
}

contract Factory {
    function make(uint256 v) public payable returns (uint256) {
        try new Created{value: 1, salt: bytes32(uint256(7))}(v) returns (Created c) {
            return c.x();
        } catch {
            return 0;
        }
    }
}
