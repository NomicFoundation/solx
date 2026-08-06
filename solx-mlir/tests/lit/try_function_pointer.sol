// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*state_pointer.*}}
// CHECK:   %[[CALL:.*]]:2 = sol.ext_icall %{{.*}}() gas %{{.*}} value %{{.*}} {try_call} : !sol.ext_func_ref<() -> ui256>, () -> (i1, ui256)
// CHECK:   sol.try %[[CALL]]#0 {
// CHECK:   } panic {
// CHECK-NEXT: } error {
// CHECK-NEXT: } fallback {
// CHECK-NEXT:   sol.constant 0 : ui8

// CHECK: sol.func @{{.*guarded.*}}
// CHECK:   sol.ext_icall %{{.*}}(%{{.*}}) gas %{{.*}} value %{{.*}} {try_call} : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*guarded_view.*}}
// CHECK:   sol.ext_icall %{{.*}}(%{{.*}}) gas %{{.*}} value %{{.*}} {static_call, try_call} : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)

contract C {
    function() external returns (uint256) stored;

    function state_pointer() public returns (uint256) {
        try stored() returns (uint256 r) {
            return r;
        } catch {
            return 0;
        }
    }

    function guarded(function(uint256) external returns (uint256) p) public returns (uint256) {
        try p(1) returns (uint256 r) {
            return r;
        } catch {
            return 0;
        }
    }

    function guarded_view(function(uint256) external view returns (uint256) p)
        public
        view
        returns (uint256)
    {
        try p(2) returns (uint256 r) {
            return r;
        } catch {
            return 0;
        }
    }
}
