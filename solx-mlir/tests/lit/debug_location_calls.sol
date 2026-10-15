// RUN: slang --emit-mlir=sol --debug-info --debug-info-runtime %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: #[[CALLER:loc[0-9]*]] = loc("{{.*}}debug_location_calls.sol":95:5)
// CHECK: #[[TWICE:loc[0-9]*]] = loc("{{.*}}debug_location_calls.sol":74:1)

// CHECK: sol.func @{{.*base.*}}()
// CHECK:   sol.call @{{.*base.*}}() {{.*}} loc(#[[SUPER_CALL:loc[0-9]*]])
// CHECK: sol.func @{{.*base.*}}()
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[BASE_RETURN:loc[0-9]*]])
// CHECK: sol.func @{{.*creates.*}}()
// CHECK:   sol.new "{{.*}}:D" {{.*}} loc(#[[NEW:loc[0-9]*]])
// CHECK: sol.func @{{.*delegates.*}}(
// CHECK:   sol.call @{{.*halve.*}}(%{{.*}}) {{.*}} loc(#[[ATTACHED_CALL:loc[0-9]*]])
// CHECK:   sol.call @{{.*halve.*}}(%{{.*}}) {{.*}} loc(#[[LIBRARY_CALL:loc[0-9]*]])
// CHECK: sol.func @{{.*halve.*}}(
// CHECK:   sol.cdiv %{{.*}}, %{{.*}} : ui256 loc(#[[QUOTIENT:loc[0-9]*]])
// CHECK: sol.func @{{.*emits.*}}(%arg0: ui256 loc("{{.*}}debug_location_calls.sol":103:5))
// CHECK:   sol.store %arg0, %{{.*}} loc(#[[AMOUNT:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[READ_AMOUNT:loc[0-9]*]])
// CHECK:   sol.emit "Moved(uint256)" {{.*}} loc(#[[EMIT:loc[0-9]*]])
// CHECK: sol.func @{{.*forwards.*}}(
// CHECK:   sol.cast %{{.*}} : ui8 to ui256 loc(#[[OPTION:loc[0-9]*]])
// CHECK: sol.func @{{.*points.*}}(
// CHECK:   sol.func_constant @{{.*halve.*}} {{.*}} loc(#[[POINTER:loc[0-9]*]])
// CHECK:   sol.icall %{{.*}} loc(#[[POINTER_CALL:loc[0-9]*]])
// CHECK: sol.func @{{.*reenters.*}}()
// CHECK:   sol.this {{.*}} loc(#[[EXTERNAL_CALL:loc[0-9]*]])
// CHECK:   sol.ext_call "{{.*run.*}}"({{.*}} loc(#[[EXTERNAL_CALL]])
// CHECK: sol.func @{{.*reverts.*}}()
// CHECK:   sol.revert loc(#[[REVERT:loc[0-9]*]])
// CHECK: sol.func @{{.*run.*}}(%arg0: ui256 loc("{{.*}}debug_location_calls.sol":95:5))
// CHECK:   sol.store %arg0, %{{.*}} loc(#[[A:loc[0-9]*]])
// CHECK:   sol.constant 1 : ui8 loc(#[[ONE:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[READ_A:loc[0-9]*]])
// CHECK:   sol.call @{{.*twice.*}} loc(#[[CALL:loc[0-9]*]])
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256 loc(#[[CALL]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[RETURN:loc[0-9]*]])
// CHECK: } loc(#[[CALLER_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*twice.*}}(%arg0: ui256 loc("{{.*}}debug_location_calls.sol":74:1))
// CHECK:   sol.store %arg0, %{{.*}} loc(#[[X:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[PRODUCT:loc[0-9]*]])
// CHECK:   sol.cmul %{{.*}}, %{{.*}} : ui256 loc(#[[PRODUCT]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[BODY_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[TWICE_FN:loc[0-9]*]])

// CHECK-DAG: #[[A]] = loc("{{.*}}debug_location_calls.sol":95:18)
// CHECK-DAG: #[[CALL]] = loc("{{.*}}debug_location_calls.sol":96:16)
// CHECK-DAG: #[[READ_A]] = loc("{{.*}}debug_location_calls.sol":96:22)
// CHECK-DAG: #[[ONE]] = loc("{{.*}}debug_location_calls.sol":96:27)
// CHECK-DAG: #[[RETURN]] = loc("{{.*}}debug_location_calls.sol":96:9)
// CHECK-DAG: #[[X]] = loc("{{.*}}debug_location_calls.sol":74:16)
// CHECK-DAG: #[[PRODUCT]] = loc("{{.*}}debug_location_calls.sol":75:12)
// CHECK-DAG: #[[BODY_RETURN]] = loc("{{.*}}debug_location_calls.sol":75:5)
// CHECK-DAG: #[[AMOUNT]] = loc("{{.*}}debug_location_calls.sol":103:20)
// CHECK-DAG: #[[READ_AMOUNT]] = loc("{{.*}}debug_location_calls.sol":104:20)
// CHECK-DAG: #[[EMIT]] = loc("{{.*}}debug_location_calls.sol":104:9)
// CHECK-DAG: #[[REVERT]] = loc("{{.*}}debug_location_calls.sol":100:9)
// CHECK-DAG: #[[SUPER_CALL]] = loc("{{.*}}debug_location_calls.sol":120:16)
// CHECK-DAG: #[[BASE_RETURN]] = loc("{{.*}}debug_location_calls.sol":86:9)
// CHECK-DAG: #[[NEW]] = loc("{{.*}}debug_location_calls.sol":112:16)
// CHECK-DAG: #[[ATTACHED_CALL]] = loc("{{.*}}debug_location_calls.sol":116:29)
// CHECK-DAG: #[[LIBRARY_CALL]] = loc("{{.*}}debug_location_calls.sol":116:16)
// CHECK-DAG: #[[QUOTIENT]] = loc("{{.*}}debug_location_calls.sol":80:16)
// CHECK-DAG: #[[POINTER]] = loc("{{.*}}debug_location_calls.sol":124:60)
// CHECK-DAG: #[[POINTER_CALL]] = loc("{{.*}}debug_location_calls.sol":125:16)
// CHECK-DAG: #[[EXTERNAL_CALL]] = loc("{{.*}}debug_location_calls.sol":108:16)
// CHECK-DAG: #[[OPTION]] = loc("{{.*}}debug_location_calls.sol":129:9)

// CHECK-DAG: #[[CALLER_FN]] = loc(fused<#[[CALLER_SP:di_subprogram[0-9]*]]>[#[[CALLER]]])
// CHECK-DAG: #[[CALLER_SP]] = #llvm.di_subprogram<{{.*}}name = "run", linkageName = "run", {{.*}}type = #di_subroutine_type>
// CHECK-DAG: #[[TWICE_FN]] = loc(fused<#[[TWICE_SP:di_subprogram[0-9]*]]>[#[[TWICE]]])
// CHECK-DAG: #[[TWICE_SP]] = #llvm.di_subprogram<{{.*}}name = "twice", linkageName = "twice", {{.*}}type = #di_subroutine_type>

function twice(uint256 x) pure returns (uint256) {
    return x * 2;
}

library L {
    function halve(uint256 x) internal pure returns (uint256) {
        return x / 2;
    }
}

abstract contract B {
    function base() public pure virtual returns (uint256) {
        return 1;
    }
}

contract D {}

contract C is B {
    using L for uint256;

    function run(uint256 a) public pure returns (uint256) {
        return twice(a) + 1;
    }

    function reverts() public pure {
        revert();
    }

    function emits(uint256 a) public {
        emit Moved(a);
    }

    function reenters() public view returns (uint256) {
        return this.run(1);
    }

    function creates() public returns (D) {
        return new D();
    }

    function delegates(uint256 a) public pure returns (uint256) {
        return L.halve(a) + a.halve();
    }

    function base() public pure override returns (uint256) {
        return super.base() + 1;
    }

    function points(uint256 a) public pure returns (uint256) {
        function(uint256) pure returns (uint256) pointer = L.halve;
        return pointer(a);
    }

    function forwards(uint8 g) public view {
        this.run{gas: g};
    }

    event Moved(uint256 amount);
}
