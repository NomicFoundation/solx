// RUN: slang --emit-mlir=sol --debug-info %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: #[[TWICE:loc[0-9]*]] = loc("{{.*}}debug_location_assembly.sol":81:13)

// CHECK: sol.inline_asm {
// CHECK:   yul.func @{{.*twice.*}} : (i256) -> i256 {
// CHECK:     yul.store %arg{{[0-9]+}}, %{{.*}} loc(#[[V:loc[0-9]*]])
// CHECK:     yul.constant 0 loc(#[[W:loc[0-9]*]])
// CHECK:     yul.add %{{.*}}, %{{.*}} loc(#[[DOUBLED:loc[0-9]*]])
// CHECK:     yul.store %{{.*}}, %{{.*}} loc(#[[ASSIGN_W:loc[0-9]*]])
// CHECK:     yul.cmp ugt, %{{.*}} loc(#[[GREATER:loc[0-9]*]])
// CHECK:     yul.if %{{.*}} {
// CHECK:       yul.func_return %{{.*}} loc(#[[LEAVE:loc[0-9]*]])
// CHECK:     } loc(#[[IF:loc[0-9]*]])
// CHECK:     yul.func_return %{{.*}} loc(#[[TWICE]])
// CHECK:   } loc(#[[TWICE_FN:loc[0-9]*]])
// CHECK:   sol.yul_ptr_cast %{{.*}} loc(#[[X:loc[0-9]*]])
// CHECK:   yul.func_call @{{.*twice.*}}(%{{.*}}) {{.*}} loc(#[[CALL:loc[0-9]*]])
// CHECK:   yul.store %{{.*}}, %{{.*}} loc(#[[LET_Y:loc[0-9]*]])
// CHECK:   yul.store %{{.*}}, %{{.*}} loc(#[[LET_I:loc[0-9]*]])
// CHECK:   yul.for cond {
// CHECK:     yul.cmp ult, %{{.*}} loc(#[[LESS:loc[0-9]*]])
// CHECK:     yul.condition %{{.*}} loc(#[[FOR:loc[0-9]*]])
// CHECK:   } body {
// CHECK:     yul.continue loc(#[[CONTINUE:loc[0-9]*]])
// CHECK:     yul.break loc(#[[BREAK:loc[0-9]*]])
// CHECK:     yul.store %{{.*}}, %{{.*}} loc(#[[ASSIGN_Y:loc[0-9]*]])
// CHECK:     yul.yield loc(#[[FOR]])
// CHECK:   } step {
// CHECK:     yul.store %{{.*}}, %{{.*}} loc(#[[STEP:loc[0-9]*]])
// CHECK:     yul.yield loc(#[[FOR]])
// CHECK:   } loc(#[[FOR]])
// CHECK:   yul.switch %{{.*}} : i256
// CHECK:   case 0 {
// CHECK:     yul.constant 1 loc(#[[ONE:loc[0-9]*]])
// CHECK:     yul.store %{{.*}}, %{{.*}} loc(#[[CASE:loc[0-9]*]])
// CHECK:     yul.yield loc(#[[SWITCH:loc[0-9]*]])
// CHECK:   default {
// CHECK:     yul.load %{{.*}} loc(#[[POPPED:loc[0-9]*]])
// CHECK:     yul.yield loc(#[[SWITCH]])
// CHECK:   } loc(#[[SWITCH]])
// CHECK:   sol.yul_state_var_offset @{{.*stored.*}} loc(#[[OFFSET:loc[0-9]*]])
// CHECK:   sol.yul_state_var_slot @{{.*stored.*}} loc(#[[SLOT:loc[0-9]*]])
// CHECK:   yul.store %{{.*}}, %{{.*}} loc(#[[ASSIGN_R:loc[0-9]*]])
// CHECK: } loc(#[[ASSEMBLY:loc[0-9]*]])

// CHECK-DAG: #[[ASSEMBLY]] = loc("{{.*}}debug_location_assembly.sol":80:9)
// CHECK-DAG: #[[V]] = loc("{{.*}}debug_location_assembly.sol":81:28)
// CHECK-DAG: #[[W]] = loc("{{.*}}debug_location_assembly.sol":81:34)
// CHECK-DAG: #[[ASSIGN_W]] = loc("{{.*}}debug_location_assembly.sol":82:17)
// CHECK-DAG: #[[DOUBLED]] = loc("{{.*}}debug_location_assembly.sol":82:22)
// CHECK-DAG: #[[IF]] = loc("{{.*}}debug_location_assembly.sol":83:17)
// CHECK-DAG: #[[GREATER]] = loc("{{.*}}debug_location_assembly.sol":83:20)
// CHECK-DAG: #[[LEAVE]] = loc("{{.*}}debug_location_assembly.sol":84:21)
// CHECK-DAG: #[[LET_Y]] = loc("{{.*}}debug_location_assembly.sol":87:13)
// CHECK-DAG: #[[CALL]] = loc("{{.*}}debug_location_assembly.sol":87:22)
// CHECK-DAG: #[[X]] = loc("{{.*}}debug_location_assembly.sol":87:28)
// CHECK-DAG: #[[FOR]] = loc("{{.*}}debug_location_assembly.sol":88:13)
// CHECK-DAG: #[[LET_I]] = loc("{{.*}}debug_location_assembly.sol":88:19)
// CHECK-DAG: #[[LESS]] = loc("{{.*}}debug_location_assembly.sol":88:32)
// CHECK-DAG: #[[STEP]] = loc("{{.*}}debug_location_assembly.sol":88:43)
// CHECK-DAG: #[[CONTINUE]] = loc("{{.*}}debug_location_assembly.sol":90:21)
// CHECK-DAG: #[[BREAK]] = loc("{{.*}}debug_location_assembly.sol":93:21)
// CHECK-DAG: #[[ASSIGN_Y]] = loc("{{.*}}debug_location_assembly.sol":95:17)
// CHECK-DAG: #[[SWITCH]] = loc("{{.*}}debug_location_assembly.sol":97:13)
// CHECK-DAG: #[[CASE]] = loc("{{.*}}debug_location_assembly.sol":99:17)
// CHECK-DAG: #[[ONE]] = loc("{{.*}}debug_location_assembly.sol":99:22)
// CHECK-DAG: #[[POPPED]] = loc("{{.*}}debug_location_assembly.sol":102:21)
// CHECK-DAG: #[[OFFSET]] = loc("{{.*}}debug_location_assembly.sol":104:34)
// CHECK-DAG: #[[SLOT]] = loc("{{.*}}debug_location_assembly.sol":104:21)
// CHECK-DAG: #[[ASSIGN_R]] = loc("{{.*}}debug_location_assembly.sol":105:13)

// CHECK-DAG: #[[TWICE_FN]] = loc(fused<#[[TWICE_SP:di_subprogram[0-9]*]]>[#[[TWICE]]])
// CHECK-DAG: #[[TWICE_SP]] = #llvm.di_subprogram<{{.*}}name = "twice", linkageName = "twice", file = #di_file, line = 81, scopeLine = 81, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>

contract C {
    uint256 stored;

    function f(uint256 x) public pure returns (uint256 r) {
        assembly {
            function twice(v) -> w {
                w := add(v, v)
                if gt(w, 100) {
                    leave
                }
            }
            let y := twice(x)
            for { let i := 0 } lt(i, 3) { i := add(i, 1) } {
                if eq(i, 1) {
                    continue
                }
                if eq(i, 2) {
                    break
                }
                y := add(y, i)
            }
            switch y
            case 0 {
                y := 1
            }
            default {
                pop(y)
            }
            pop(add(stored.slot, stored.offset))
            r := y
        }
    }
}
