// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc's print-init substitutes `sol.timestamp` for module member accesses and crashes on a
// renamed self-import, so this is solx-only.

// CHECK: sol.func @{{.*chained.*}}() -> ui256 attributes {{.*}}selector = 1955792327 : i32
// CHECK:   %[[CK:.*]] = sol.constant 11 : ui8
// CHECK:   %[[CV:.*]] = sol.cast %[[CK]] : ui8 to ui256
// CHECK:   sol.return %[[CV]] : ui256

// CHECK: sol.func @{{.*renamed.*}}() -> ui256 attributes {{.*}}selector = -1753621920 : i32
// CHECK:   %[[RA:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[RC:.*]] = sol.call @{{.*freeTriple.*}}(%[[RA]]) : (ui256) -> ui256
// CHECK:   sol.constant 11 : ui8
// CHECK:   sol.return

// CHECK: sol.func @{{.*starred.*}}() -> ui256 attributes {{.*}}selector = -1638182610 : i32
// CHECK:   %[[SK:.*]] = sol.constant 11 : ui8
// CHECK:   %[[SV:.*]] = sol.cast %[[SK]] : ui8 to ui256
// CHECK:   sol.return %[[SV]] : ui256

import "./module_members.sol" as M;
import {FREE_K as RENAMED_K, freeTriple} from "./module_members.sol";
import * as S from "./module_members.sol";

uint256 constant FREE_K = 11;

function freeTriple(uint256 x) pure returns (uint256) {
    return x * 3;
}

contract ModuleMembers {
    function chained() public pure returns (uint256) {
        return M.M.M.FREE_K;
    }

    function renamed() public pure returns (uint256) {
        return RENAMED_K + freeTriple(2);
    }

    function starred() public pure returns (uint256) {
        return S.FREE_K;
    }
}
