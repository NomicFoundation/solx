// RUN: slang --emit-mlir=sol --debug-info %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: #[[ARITHMETIC:loc[0-9]*]] = loc("{{.*}}debug_location_arithmetic.sol":96:5)
// CHECK: #[[BITS:loc[0-9]*]] = loc("{{.*}}debug_location_arithmetic.sol":102:5)
// CHECK: #[[LOGIC:loc[0-9]*]] = loc("{{.*}}debug_location_arithmetic.sol":120:5)

// CHECK: sol.func @{{.*arithmetic.*}}(%arg0: ui256 loc("{{.*}}debug_location_arithmetic.sol":96:5), %arg1: ui256 loc("{{.*}}debug_location_arithmetic.sol":96:5))
// CHECK:   sol.store %arg0, %{{.*}} loc(#[[A:loc[0-9]*]])
// CHECK:   sol.store %arg1, %{{.*}} loc(#[[B:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}, %{{.*}} loc(#[[R:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[READ_B:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[SUM:loc[0-9]*]])
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256 loc(#[[SUM]])
// CHECK:   sol.store %{{.*}}, %{{.*}} loc(#[[ASSIGN_SUM:loc[0-9]*]])
// CHECK:   sol.cmul %{{.*}}, %{{.*}} : ui256 loc(#[[PRODUCT:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}, %{{.*}} loc(#[[ASSIGN_PRODUCT:loc[0-9]*]])
// CHECK:   sol.csub %{{.*}}, %{{.*}} : ui256 loc(#[[DIFFERENCE:loc[0-9]*]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[ARITHMETIC_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[ARITHMETIC_FN:loc[0-9]*]])

// CHECK: sol.func @{{.*bits.*}}(
// CHECK:   sol.and %{{.*}}, %{{.*}} : ui256 loc(#[[AND:loc[0-9]*]])
// CHECK:   sol.or %{{.*}}, %{{.*}} : ui256 loc(#[[OR:loc[0-9]*]])
// CHECK:   sol.xor %{{.*}}, %{{.*}} : ui256 loc(#[[XOR:loc[0-9]*]])
// CHECK:   sol.shl %{{.*}}, %{{.*}} : ui256, ui8 loc(#[[SHIFT_LEFT:loc[0-9]*]])
// CHECK:   sol.shr %{{.*}}, %{{.*}} : ui256, ui8 loc(#[[SHIFT_RIGHT:loc[0-9]*]])
// CHECK:   sol.constant 2 : ui8 loc(#[[EXPONENT:loc[0-9]*]])
// CHECK:   sol.cexp %{{.*}} loc(#[[POWER:loc[0-9]*]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[BITS_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[BITS_FN:loc[0-9]*]])

// CHECK: sol.func @{{.*compares.*}}(
// CHECK:   sol.cmp eq, %{{.*}} loc(#[[EQUAL:loc[0-9]*]])
// CHECK:   sol.cmp ne, %{{.*}} loc(#[[NOT_EQUAL:loc[0-9]*]])
// CHECK:   sol.constant true loc(#[[TRUE:loc[0-9]*]])
// CHECK:   sol.constant false loc(#[[FALSE:loc[0-9]*]])

// CHECK: sol.func @{{.*converts.*}}(
// CHECK:   sol.cast %{{.*}} : ui256 to ui8 loc(#[[TO_UINT8:loc[0-9]*]])
// CHECK:   sol.cast %{{.*}} : ui256 to si256 loc(#[[TO_INT256:loc[0-9]*]])
// CHECK:   sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32> loc(#[[TO_BYTES32:loc[0-9]*]])

// CHECK: sol.func @{{.*logic.*}}(
// CHECK:   sol.load %{{.*}} loc(#[[FIRST_P:loc[0-9]*]])
// CHECK:   sol.constant false loc(#[[FIRST_P]])
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.load %{{.*}} loc(#[[Q:loc[0-9]*]])
// CHECK:   } else {
// CHECK:   } loc(#[[FIRST_P]])
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.yield loc(#[[FIRST_P]])
// CHECK:   } else {
// CHECK:     sol.load %{{.*}} loc(#[[LAST_P:loc[0-9]*]])
// CHECK:   } loc(#[[FIRST_P]])
// CHECK:   sol.return %{{.*}} : i1 loc(#[[LOGIC_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[LOGIC_FN:loc[0-9]*]])

// CHECK-DAG: #[[A]] = loc("{{.*}}debug_location_arithmetic.sol":96:25)
// CHECK-DAG: #[[B]] = loc("{{.*}}debug_location_arithmetic.sol":96:36)
// CHECK-DAG: #[[R]] = loc("{{.*}}debug_location_arithmetic.sol":96:68)
// CHECK-DAG: #[[READ_B]] = loc("{{.*}}debug_location_arithmetic.sol":97:17)
// CHECK-DAG: #[[SUM]] = loc("{{.*}}debug_location_arithmetic.sol":97:13)
// CHECK-DAG: #[[ASSIGN_SUM]] = loc("{{.*}}debug_location_arithmetic.sol":97:9)
// CHECK-DAG: #[[PRODUCT]] = loc("{{.*}}debug_location_arithmetic.sol":98:13)
// CHECK-DAG: #[[ASSIGN_PRODUCT]] = loc("{{.*}}debug_location_arithmetic.sol":98:9)
// CHECK-DAG: #[[DIFFERENCE]] = loc("{{.*}}debug_location_arithmetic.sol":99:16)
// CHECK-DAG: #[[ARITHMETIC_RETURN]] = loc("{{.*}}debug_location_arithmetic.sol":99:9)
// CHECK-DAG: #[[AND]] = loc("{{.*}}debug_location_arithmetic.sol":103:21)
// CHECK-DAG: #[[OR]] = loc("{{.*}}debug_location_arithmetic.sol":104:13)
// CHECK-DAG: #[[XOR]] = loc("{{.*}}debug_location_arithmetic.sol":105:13)
// CHECK-DAG: #[[SHIFT_LEFT]] = loc("{{.*}}debug_location_arithmetic.sol":106:13)
// CHECK-DAG: #[[SHIFT_RIGHT]] = loc("{{.*}}debug_location_arithmetic.sol":107:13)
// CHECK-DAG: #[[POWER]] = loc("{{.*}}debug_location_arithmetic.sol":108:16)
// CHECK-DAG: #[[EXPONENT]] = loc("{{.*}}debug_location_arithmetic.sol":109:16)
// CHECK-DAG: #[[BITS_RETURN]] = loc("{{.*}}debug_location_arithmetic.sol":108:9)
// CHECK-DAG: #[[FIRST_P]] = loc("{{.*}}debug_location_arithmetic.sol":121:16)
// CHECK-DAG: #[[Q]] = loc("{{.*}}debug_location_arithmetic.sol":121:21)
// CHECK-DAG: #[[LAST_P]] = loc("{{.*}}debug_location_arithmetic.sol":121:26)
// CHECK-DAG: #[[LOGIC_RETURN]] = loc("{{.*}}debug_location_arithmetic.sol":121:9)
// CHECK-DAG: #[[EQUAL]] = loc("{{.*}}debug_location_arithmetic.sol":113:17)
// CHECK-DAG: #[[NOT_EQUAL]] = loc("{{.*}}debug_location_arithmetic.sol":113:25)
// CHECK-DAG: #[[TRUE]] = loc("{{.*}}debug_location_arithmetic.sol":113:33)
// CHECK-DAG: #[[FALSE]] = loc("{{.*}}debug_location_arithmetic.sol":113:39)
// CHECK-DAG: #[[TO_UINT8]] = loc("{{.*}}debug_location_arithmetic.sol":117:17)
// CHECK-DAG: #[[TO_INT256]] = loc("{{.*}}debug_location_arithmetic.sol":117:27)
// CHECK-DAG: #[[TO_BYTES32]] = loc("{{.*}}debug_location_arithmetic.sol":117:38)

// CHECK-DAG: #[[ARITHMETIC_FN]] = loc(fused<#[[ARITHMETIC_SP:di_subprogram[0-9]*]]>[#[[ARITHMETIC]]])
// CHECK-DAG: #[[ARITHMETIC_SP]] = #llvm.di_subprogram<{{.*}}name = "arithmetic", linkageName = "arithmetic", {{.*}}type = #di_subroutine_type>
// CHECK-DAG: #[[BITS_FN]] = loc(fused<#[[BITS_SP:di_subprogram[0-9]*]]>[#[[BITS]]])
// CHECK-DAG: #[[BITS_SP]] = #llvm.di_subprogram<{{.*}}name = "bits", linkageName = "bits", {{.*}}type = #di_subroutine_type>
// CHECK-DAG: #[[LOGIC_FN]] = loc(fused<#[[LOGIC_SP:di_subprogram[0-9]*]]>[#[[LOGIC]]])
// CHECK-DAG: #[[LOGIC_SP]] = #llvm.di_subprogram<{{.*}}name = "logic", linkageName = "logic", {{.*}}type = #di_subroutine_type>

contract C {
    function arithmetic(uint256 a, uint256 b) public pure returns (uint256 r) {
        r = a + b;
        r = r * 2;
        return r - 1;
    }

    function bits(uint256 a, uint256 b) public pure returns (uint256) {
        uint256 x = a & b;
        x = x | b;
        x = x ^ a;
        x = x << 3;
        x = x >> 1;
        return x
            ** 2;
    }

    function compares(uint256 a, uint256 b) public pure returns (bool, bool, bool, bool) {
        return (a == b, a != b, true, false);
    }

    function converts(uint256 a) public pure returns (uint8, int256, bytes32) {
        return (uint8(a), int256(a), bytes32(a));
    }

    function logic(bool p, bool q) public pure returns (bool) {
        return p && q || p;
    }
}
