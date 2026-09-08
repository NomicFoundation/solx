// RUN: solx --emit-mlir=sol %modifier_base_alias/main.sol %modifier_base_alias/base.sol | FileCheck %s
// RUN: solc --mlir-action=print-init %modifier_base_alias/main.sol %modifier_base_alias/base.sol 2>/dev/null | FileCheck %s

// A base named through an import alias in the constructor's modifier list is a base-constructor
// argument list, evaluated at the call to that constructor.
// CHECK: sol.contract @{{.*}}Test
// CHECK:   sol.func @{{.*}}() attributes {{.*}}kind = #{{.*}}Constructor
// CHECK:     sol.constant 7 : ui8
// CHECK:     sol.call @[[BASE_CONSTRUCTOR:.*]](%{{.*}}) : (ui256) -> ()
// CHECK:   sol.func @[[BASE_CONSTRUCTOR]](%arg0: ui256) attributes {{{(id = [0-9]+ : i64, )?}}state_mutability
