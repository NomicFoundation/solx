// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc's print-init substitutes `sol.timestamp` for module member accesses, so this is solx-only.

// CHECK: sol.func @{{.*parenthesized.*}}() -> ui256 attributes {{.*}}selector = -923061170 : i32
// CHECK:   %[[PK:.*]] = sol.constant 11 : ui8
// CHECK:   %[[PV:.*]] = sol.cast %[[PK]] : ui8 to ui256
// CHECK:   sol.return %[[PV]] : ui256

// CHECK: sol.func @{{.*conditional.*}}() -> ui256 attributes {{.*}}selector = -1619768731 : i32
// CHECK:   %[[CP:.*]] = sol.addr_of @{{.*flag.*}} : !sol.ptr<i1, Storage>
// CHECK:   sol.store %true, %[[CP]] : i1, !sol.ptr<i1, Storage>
// CHECK:   sol.constant 11 : ui8
// CHECK:   sol.return

// CHECK: sol.func @{{.*chained.*}}() -> ui256 attributes {{.*}}selector = 1955792327 : i32
// CHECK:   sol.store %true, %{{.*}} : i1, !sol.ptr<i1, Storage>
// CHECK:   sol.constant 11 : ui8
// CHECK:   sol.return

// CHECK: sol.func @{{.*called.*}}() -> ui256 attributes {{.*}}selector = 1358542541 : i32
// CHECK:   %[[AV:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.store %true, %{{.*}} : i1, !sol.ptr<i1, Storage>
// CHECK:   %[[AF:.*]] = sol.func_constant @{{.*freeHalf.*}} : !sol.func_ref<(ui256) -> ui256>
// CHECK:   %[[RV:.*]] = sol.icall %[[AF]](%[[AV]]) : !sol.func_ref<(ui256) -> ui256>, (ui256) -> ui256
// CHECK:   sol.return %[[RV]] : ui256

// CHECK: sol.func @{{.*tried.*}}() -> ui256 attributes {{.*}}selector = -553464963 : i32
// CHECK:   sol.store %true, %{{.*}} : i1, !sol.ptr<i1, Storage>
// CHECK:   %[[LA:.*]] = sol.lib_addr "{{[^"]*}}Halver" : !sol.address
// CHECK:   sol.ext_call "{{.*half.*}}"(%{{.*}}) at %[[LA]] gas %{{.*}}try_call{{.*}} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: sol.func @{{.*designated.*}}() -> ui256 attributes {{.*}}selector = -1019040149 : i32
// CHECK:   %[[DV:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.store %true, %{{.*}} : i1, !sol.ptr<i1, Storage>
// CHECK:   %[[DF:.*]] = sol.func_constant @{{.*freeHalf.*}} : !sol.func_ref<(ui256) -> ui256>
// CHECK:   %[[DR:.*]] = sol.icall %[[DF]](%[[DV]]) : !sol.func_ref<(ui256) -> ui256>, (ui256) -> ui256
// CHECK:   sol.return %[[DR]] : ui256

// CHECK: sol.func @{{.*qualified.*}}() -> ui256 attributes {{.*}}selector = -228858638 : i32
// CHECK:   %[[QV:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[QR:.*]] = sol.call @{{.*halve.*}}(%[[QV]]) : (ui256) -> ui256
// CHECK:   sol.return %[[QR]] : ui256

// CHECK: sol.func @{{.*wrapped.*}}() -> ui256 attributes {{.*}}selector = 1357319496 : i32
// CHECK:   %[[WV:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.return %[[WV]] : ui256

// CHECK: sol.func @{{.*stray.*}}() -> i1 attributes {{.*}}selector = -903043920 : i32
// CHECK:   sol.store %true, %{{.*}} : i1, !sol.ptr<i1, Storage>
// CHECK:   %[[SP:.*]] = sol.addr_of @{{.*flag.*}} : !sol.ptr<i1, Storage>
// CHECK:   %[[SV:.*]] = sol.load %[[SP]] : !sol.ptr<i1, Storage>, i1
// CHECK:   sol.return %[[SV]] : i1

import "./module_values.sol" as M;

uint256 constant FREE_K = 11;

type Cost is uint256;

function freeHalf(uint256 x) pure returns (uint256) {
    return x / 2;
}

library Halver {
    function half(uint256 x) public pure returns (uint256) {
        return x / 2;
    }
}

contract ModuleValues {
    bool flag;

    function parenthesized() public pure returns (uint256) {
        return (M).FREE_K;
    }

    function conditional() public returns (uint256) {
        return ((flag = true) ? M : M).FREE_K;
    }

    function chained() public returns (uint256) {
        return ((flag = true) ? M : M).M.FREE_K;
    }

    function called() public returns (uint256) {
        return ((flag = true) ? M : M).freeHalf(8);
    }

    function tried() public returns (uint256) {
        try ((flag = true) ? M : M).Halver.half(4) returns (uint256 v) {
            return v;
        } catch {
            return 0;
        }
    }

    function designated() public returns (uint256) {
        return (((flag = true) ? M : M).freeHalf)(8);
    }

    function halve(uint256 x) internal pure returns (uint256) {
        return x / 2;
    }

    function qualified() public returns (uint256) {
        return ((flag = true) ? M : M).ModuleValues.halve(6);
    }

    function wrapped() public returns (uint256) {
        return Cost.unwrap(((flag = true) ? M : M).Cost.wrap(6));
    }

    function stray() public returns (bool) {
        M;
        M.ModuleValues;
        ((flag = true) ? M : M).ModuleValues;
        return flag;
    }
}
