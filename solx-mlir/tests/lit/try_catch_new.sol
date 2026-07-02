// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `try new C(args) { } catch ... { }` over a contract creation: the creation is marked `try` so the
// surrounding `sol.try` receives a success status (the created contract's address != 0) and runs a
// catch handler on a revert instead of propagating it. Both backends emit the same shape: a
// `sol.new ... try`, two `sol.address_cast`s (contract -> address -> ui160), a `sol.cmp ne` against
// zero, and a `sol.try` with success/panic/error/fallback regions.

contract C {
    function f(uint v) public returns (uint) {
        try new Callable(v) returns (Callable) {
            return 0;
        } catch Error(string memory) {
            return 1;
        } catch Panic(uint) {
            return 2;
        } catch (bytes memory) {
            return 3;
        }
    }
}

contract Callable {
    uint public x;
    constructor(uint v) { require(v != 0); x = v; }
}

// CHECK:      sol.new "Callable{{.*}}" value = %{{.*}} ctor(%{{.*}} : ui256) try : !sol.contract<"Callable{{.*}}">
// CHECK-NEXT: sol.address_cast %{{.*}} : !sol.contract<"Callable{{.*}}"> to !sol.address
// CHECK-NEXT: sol.address_cast %{{.*}} : !sol.address to ui160
// CHECK:      %[[ST:.*]] = sol.cmp ne, %{{.*}}, %{{.*}} : ui160
// CHECK:      sol.try %[[ST]] {
// CHECK:      } panic {
// CHECK:      } error {
// CHECK:      } fallback {
