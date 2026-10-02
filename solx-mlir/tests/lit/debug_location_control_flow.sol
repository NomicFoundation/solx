// RUN: solx --emit-mlir=sol --debug-info %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK-DAG: #[[BRANCHES:loc[0-9]*]] = loc("{{.*}}debug_location_control_flow.sol":158:5)
// CHECK-DAG: #[[LOOPS:loc[0-9]*]] = loc("{{.*}}debug_location_control_flow.sol":166:5)
// CHECK-DAG: #[[UNBOUNDED:loc[0-9]*]] = loc("{{.*}}debug_location_control_flow.sol":180:5)

// CHECK: sol.func @{{.*attempts.*}}(
// CHECK:   sol.try %{{.*}} {
// CHECK:     sol.return %{{.*}} : ui256 loc(#[[TRY_RETURN:loc[0-9]*]])
// CHECK:   } fallback {
// CHECK:     sol.return %{{.*}} : ui256 loc(#[[CATCH_RETURN:loc[0-9]*]])
// CHECK:   } loc(#[[TRY:loc[0-9]*]])

// CHECK: sol.func @{{.*branches.*}}(%arg0: ui256 loc("{{.*}}debug_location_control_flow.sol":158:5))
// CHECK:   sol.store %arg0, %{{.*}} loc(#[[X:loc[0-9]*]])
// CHECK:   sol.constant 10 : ui8 loc(#[[TEN:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[GREATER:loc[0-9]*]])
// CHECK:   sol.cmp gt, %{{.*}} loc(#[[GREATER]])
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.return %{{.*}} : ui256 loc(#[[RETURN_ONE:loc[0-9]*]])
// CHECK:   } else {
// CHECK:     sol.return %{{.*}} : ui256 loc(#[[RETURN_ZERO:loc[0-9]*]])
// CHECK:   } loc(#[[IF:loc[0-9]*]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[BRANCHES_CLOSE:loc[0-9]*]])
// CHECK: } loc(#[[BRANCHES_FN:loc[0-9]*]])

// CHECK: sol.func @{{.*chooses.*}}(
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.call @{{.*branches.*}} loc(#[[CHOOSE_THEN:loc[0-9]*]])
// CHECK:   } else {
// CHECK:     sol.call @{{.*branches.*}} loc(#[[CHOOSE_ELSE:loc[0-9]*]])
// CHECK:   } loc(#[[CHOOSE:loc[0-9]*]])

// CHECK: sol.func @{{.*fails.*}}(
// CHECK:   sol.load %{{.*}} loc(#[[CODE:loc[0-9]*]])
// CHECK:   sol.revert "Failed(uint256)" {{.*}} loc(#[[FAILED:loc[0-9]*]])

// CHECK: sol.func @{{.*loops.*}}(%arg0: ui256 loc("{{.*}}debug_location_control_flow.sol":166:5))
// CHECK:   sol.store %arg0, %{{.*}} loc(#[[N:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}, %{{.*}} loc(#[[LOOPS_RETURN:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}, %{{.*}} loc(#[[SUM:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}, %{{.*}} loc(#[[INIT:loc[0-9]*]])
// CHECK:   sol.for cond {
// CHECK:     sol.cmp lt, %{{.*}} loc(#[[LESS:loc[0-9]*]])
// CHECK:     sol.condition %{{.*}} loc(#[[FOR:loc[0-9]*]])
// CHECK:   } body {
// CHECK:     sol.cadd %{{.*}}, %{{.*}} : ui256 loc(#[[FOR_ADD:loc[0-9]*]])
// CHECK:     sol.store %{{.*}}, %{{.*}} loc(#[[FOR_ASSIGN:loc[0-9]*]])
// CHECK:     sol.yield loc(#[[FOR]])
// CHECK:   } step {
// CHECK:     sol.cadd %{{.*}}, %{{.*}} : ui256 loc(#[[STEP:loc[0-9]*]])
// CHECK:     sol.yield loc(#[[FOR]])
// CHECK:   } loc(#[[FOR]])
// CHECK:   sol.while {
// CHECK:     sol.cmp gt, %{{.*}} loc(#[[WHILE_GREATER:loc[0-9]*]])
// CHECK:     sol.condition %{{.*}} loc(#[[WHILE:loc[0-9]*]])
// CHECK:   } do {
// CHECK:     sol.csub %{{.*}}, %{{.*}} : ui256 loc(#[[WHILE_SUB:loc[0-9]*]])
// CHECK:     sol.yield loc(#[[WHILE]])
// CHECK:   } loc(#[[WHILE]])
// CHECK:   sol.do {
// CHECK:     sol.cadd %{{.*}}, %{{.*}} : ui256 loc(#[[DO_ADD:loc[0-9]*]])
// CHECK:     sol.yield loc(#[[DO:loc[0-9]*]])
// CHECK:   } while {
// CHECK:     sol.cmp lt, %{{.*}} loc(#[[DO_LESS:loc[0-9]*]])
// CHECK:     sol.condition %{{.*}} loc(#[[DO]])
// CHECK:   } loc(#[[DO]])
// CHECK:   sol.load %{{.*}} loc(#[[READ_SUM:loc[0-9]*]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[RETURN:loc[0-9]*]])
// CHECK: } loc(#[[LOOPS_FN:loc[0-9]*]])

// CHECK: sol.func @{{.*picks.*}}(
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.load %{{.*}} loc(#[[PICK_A:loc[0-9]*]])
// CHECK:   } else {
// CHECK:     sol.load %{{.*}} loc(#[[PICK_B:loc[0-9]*]])
// CHECK:   } loc(#[[PICK:loc[0-9]*]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[PICK_RETURN:loc[0-9]*]])

// CHECK: sol.func @{{.*recovers.*}}(
// CHECK:   sol.ext_call {{.*}} loc(#[[TRY_CALL:loc[0-9]*]])
// CHECK:   sol.try %{{.*}} {
// CHECK:     sol.store %{{.*}}, %{{.*}} loc(#[[TRY_VALUE:loc[0-9]*]])
// CHECK:     sol.return %{{.*}} : ui256 loc(#[[TRY_VALUE_RETURN:loc[0-9]*]])
// CHECK:   } error {
// CHECK:     sol.store %arg1, %{{.*}} loc(#[[TRY_REASON:loc[0-9]*]])
// CHECK:     sol.return %{{.*}} : ui256 loc(#[[TRY_REASON_RETURN:loc[0-9]*]])

// CHECK: sol.func @{{.*skips.*}}()
// CHECK:   sol.continue loc(#[[CONTINUE:loc[0-9]*]])

// CHECK: sol.func @{{.*unbounded.*}}(%arg0: ui256 loc("{{.*}}debug_location_control_flow.sol":180:5))
// CHECK:   sol.for cond {
// CHECK:     sol.constant true loc(#[[UNBOUNDED_FOR:loc[0-9]*]])
// CHECK:     sol.condition %{{.*}} loc(#[[UNBOUNDED_FOR]])
// CHECK:   } body {
// CHECK:     sol.break loc(#[[BREAK:loc[0-9]*]])
// CHECK:   } step {
// CHECK:     sol.yield loc(#[[UNBOUNDED_FOR]])
// CHECK:   } loc(#[[UNBOUNDED_FOR]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[UNBOUNDED_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[UNBOUNDED_FN:loc[0-9]*]])

// CHECK-DAG: #[[X]] = loc("{{.*}}debug_location_control_flow.sol":158:23)
// CHECK-DAG: #[[IF]] = loc("{{.*}}debug_location_control_flow.sol":159:9)
// CHECK-DAG: #[[GREATER]] = loc("{{.*}}debug_location_control_flow.sol":159:13)
// CHECK-DAG: #[[TEN]] = loc("{{.*}}debug_location_control_flow.sol":159:17)
// CHECK-DAG: #[[RETURN_ONE]] = loc("{{.*}}debug_location_control_flow.sol":160:13)
// CHECK-DAG: #[[RETURN_ZERO]] = loc("{{.*}}debug_location_control_flow.sol":162:13)
// CHECK-DAG: #[[BRANCHES_CLOSE]] = loc("{{.*}}debug_location_control_flow.sol":164:5)
// CHECK-DAG: #[[N]] = loc("{{.*}}debug_location_control_flow.sol":166:20)
// CHECK-DAG: #[[LOOPS_RETURN]] = loc("{{.*}}debug_location_control_flow.sol":166:52)
// CHECK-DAG: #[[SUM]] = loc("{{.*}}debug_location_control_flow.sol":167:9)
// CHECK-DAG: #[[INIT]] = loc("{{.*}}debug_location_control_flow.sol":168:14)
// CHECK-DAG: #[[FOR]] = loc("{{.*}}debug_location_control_flow.sol":168:9)
// CHECK-DAG: #[[LESS]] = loc("{{.*}}debug_location_control_flow.sol":168:29)
// CHECK-DAG: #[[STEP]] = loc("{{.*}}debug_location_control_flow.sol":168:36)
// CHECK-DAG: #[[FOR_ASSIGN]] = loc("{{.*}}debug_location_control_flow.sol":169:13)
// CHECK-DAG: #[[FOR_ADD]] = loc("{{.*}}debug_location_control_flow.sol":169:19)
// CHECK-DAG: #[[WHILE]] = loc("{{.*}}debug_location_control_flow.sol":171:9)
// CHECK-DAG: #[[WHILE_GREATER]] = loc("{{.*}}debug_location_control_flow.sol":171:16)
// CHECK-DAG: #[[WHILE_SUB]] = loc("{{.*}}debug_location_control_flow.sol":172:19)
// CHECK-DAG: #[[DO]] = loc("{{.*}}debug_location_control_flow.sol":174:9)
// CHECK-DAG: #[[DO_ADD]] = loc("{{.*}}debug_location_control_flow.sol":175:19)
// CHECK-DAG: #[[DO_LESS]] = loc("{{.*}}debug_location_control_flow.sol":176:18)
// CHECK-DAG: #[[READ_SUM]] = loc("{{.*}}debug_location_control_flow.sol":177:16)
// CHECK-DAG: #[[RETURN]] = loc("{{.*}}debug_location_control_flow.sol":177:9)
// CHECK-DAG: #[[UNBOUNDED_FOR]] = loc("{{.*}}debug_location_control_flow.sol":181:9)
// CHECK-DAG: #[[BREAK]] = loc("{{.*}}debug_location_control_flow.sol":182:13)
// CHECK-DAG: #[[UNBOUNDED_RETURN]] = loc("{{.*}}debug_location_control_flow.sol":184:9)
// CHECK-DAG: #[[TRY]] = loc("{{.*}}debug_location_control_flow.sol":198:9)
// CHECK-DAG: #[[TRY_RETURN]] = loc("{{.*}}debug_location_control_flow.sol":199:13)
// CHECK-DAG: #[[CATCH_RETURN]] = loc("{{.*}}debug_location_control_flow.sol":201:13)
// CHECK-DAG: #[[TRY_CALL]] = loc("{{.*}}debug_location_control_flow.sol":206:13)
// CHECK-DAG: #[[TRY_VALUE]] = loc("{{.*}}debug_location_control_flow.sol":206:46)
// CHECK-DAG: #[[TRY_VALUE_RETURN]] = loc("{{.*}}debug_location_control_flow.sol":207:13)
// CHECK-DAG: #[[TRY_REASON]] = loc("{{.*}}debug_location_control_flow.sol":208:23)
// CHECK-DAG: #[[TRY_REASON_RETURN]] = loc("{{.*}}debug_location_control_flow.sol":209:13)
// CHECK-DAG: #[[CHOOSE]] = loc("{{.*}}debug_location_control_flow.sol":218:9)
// CHECK-DAG: #[[CHOOSE_THEN]] = loc("{{.*}}debug_location_control_flow.sol":218:13)
// CHECK-DAG: #[[CHOOSE_ELSE]] = loc("{{.*}}debug_location_control_flow.sol":218:27)
// CHECK-DAG: #[[CODE]] = loc("{{.*}}debug_location_control_flow.sol":194:23)
// CHECK-DAG: #[[FAILED]] = loc("{{.*}}debug_location_control_flow.sol":194:9)
// CHECK-DAG: #[[PICK]] = loc("{{.*}}debug_location_control_flow.sol":214:16)
// CHECK-DAG: #[[PICK_A]] = loc("{{.*}}debug_location_control_flow.sol":214:20)
// CHECK-DAG: #[[PICK_B]] = loc("{{.*}}debug_location_control_flow.sol":214:24)
// CHECK-DAG: #[[PICK_RETURN]] = loc("{{.*}}debug_location_control_flow.sol":214:9)
// CHECK-DAG: #[[CONTINUE]] = loc("{{.*}}debug_location_control_flow.sol":189:13)

// CHECK-DAG: #[[BRANCHES_FN]] = loc(fused<#[[BRANCHES_SP:di_subprogram[0-9]*]]>[#[[BRANCHES]]])
// CHECK-DAG: #[[BRANCHES_SP]] = #llvm.di_subprogram<{{.*}}name = "branches", linkageName = "branches", {{.*}}type = #di_subroutine_type>
// CHECK-DAG: #[[LOOPS_FN]] = loc(fused<#[[LOOPS_SP:di_subprogram[0-9]*]]>[#[[LOOPS]]])
// CHECK-DAG: #[[LOOPS_SP]] = #llvm.di_subprogram<{{.*}}name = "loops", linkageName = "loops", {{.*}}type = #di_subroutine_type>
// CHECK-DAG: #[[UNBOUNDED_FN]] = loc(fused<#[[UNBOUNDED_SP:di_subprogram[0-9]*]]>[#[[UNBOUNDED]]])
// CHECK-DAG: #[[UNBOUNDED_SP]] = #llvm.di_subprogram<{{.*}}name = "unbounded", linkageName = "unbounded", {{.*}}type = #di_subroutine_type>

contract C {
    function branches(uint256 x) public pure returns (uint256) {
        if (x > 10) {
            return 1;
        } else {
            return 0;
        }
    }

    function loops(uint256 n) public pure returns (uint256) {
        uint256 sum = 0;
        for (uint256 i = 0; i < n; i++) {
            sum = sum + i;
        }
        while (sum > 100) {
            sum = sum - 100;
        }
        do {
            sum = sum + 1;
        } while (sum < 10);
        return sum;
    }

    function unbounded(uint256 n) public pure returns (uint256) {
        for (;;) {
            break;
        }
        return n;
    }

    function skips() public pure {
        for (;;) {
            continue;
        }
    }

    function fails(uint256 code) public pure {
        revert Failed(code);
    }

    function attempts(C other) public pure returns (uint256) {
        try other.skips() {
            return 1;
        } catch {
            return 0;
        }
    }

    function recovers(C other) public pure returns (uint256) {
        try other.picks(true, 1, 2) returns (uint256 v) {
            return v;
        } catch Error(string memory reason) {
            return bytes(reason).length;
        }
    }

    function picks(bool c, uint256 a, uint256 b) public pure returns (uint256) {
        return c ? a : b;
    }

    function chooses(bool c) public pure {
        c ? branches(1) : branches(2);
    }

    error Failed(uint256 code);
}
