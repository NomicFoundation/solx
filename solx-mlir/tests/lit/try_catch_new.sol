// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK:      sol.new "Callable{{.*}}" value = %{{.*}} ctor(%{{.*}} : ui256) try : !sol.contract<"Callable{{.*}}">
// CHECK-NEXT: sol.address_cast %{{.*}} : !sol.contract<"Callable{{.*}}"> to !sol.address
// CHECK-NEXT: sol.address_cast %{{.*}} : !sol.address to ui160
// CHECK:      %[[ST:.*]] = sol.cmp ne, %{{.*}}, %{{.*}} : ui160
// CHECK:      sol.try %[[ST]] {
// CHECK:      } panic {
// CHECK:      } error {
// CHECK:      } fallback {

contract Callable {
    uint public x;

    constructor(uint v) { require(v != 0); x = v; }
}

contract C {
    function f(uint v) public returns (uint) {
        try new Callable(v) returns (Callable c) {
            return c.x();
        } catch Error(string memory) {
            return 1;
        } catch Panic(uint) {
            return 2;
        } catch (bytes memory) {
            return 3;
        }
    }
}
