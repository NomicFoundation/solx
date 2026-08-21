// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*chained.*}}() -> ui256 attributes {{.*}}selector = 1955792327 : i32
// CHECK:   %[[CK:.*]] = sol.constant 11 : ui8
// CHECK:   %[[CV:.*]] = sol.cast %[[CK]] : ui8 to ui256
// CHECK:   sol.return %[[CV]] : ui256

// CHECK: sol.func @{{.*renamed.*}}() -> ui256 attributes {{.*}}selector = -1753621920 : i32
// CHECK:   %[[RA:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[RC:.*]] = sol.call @{{.*freeTriple.*}}(%[[RA]]) : (ui256) -> ui256
// CHECK:   sol.return %[[RC]] : ui256

// CHECK: sol.func @{{.*starred.*}}() -> ui256 attributes {{.*}}selector = -1638182610 : i32
// CHECK:   %[[SK:.*]] = sol.constant 11 : ui8
// CHECK:   %[[SV:.*]] = sol.cast %[[SK]] : ui8 to ui256
// CHECK:   sol.return %[[SV]] : ui256

// CHECK: sol.func @{{.*plainCall.*}}() -> ui256 attributes {{.*}}selector = 1887173101 : i32
// CHECK:   %[[PA:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[PC:.*]] = sol.call @{{.*freeTriple.*}}(%[[PA]]) : (ui256) -> ui256
// CHECK:   sol.return %[[PC]] : ui256

// CHECK: sol.func @{{.*chainedCall.*}}() -> ui256 attributes {{.*}}selector = -1246699396 : i32
// CHECK:   %[[CA:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[CC:.*]] = sol.call @{{.*freeTriple.*}}(%[[CA]]) : (ui256) -> ui256
// CHECK:   sol.return %[[CC]] : ui256

// CHECK: sol.func @{{.*starredCall.*}}() -> ui256 attributes {{.*}}selector = -548888477 : i32
// CHECK:   %[[SA:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[SC:.*]] = sol.call @{{.*freeTriple.*}}(%[[SA]]) : (ui256) -> ui256
// CHECK:   sol.return %[[SC]] : ui256

// CHECK: sol.func @{{.*parenthesized.*}}() -> ui256 attributes {{.*}}selector = -923061170 : i32
// CHECK:   %[[NK:.*]] = sol.constant 11 : ui8
// CHECK:   %[[NV:.*]] = sol.cast %[[NK]] : ui8 to ui256
// CHECK:   sol.return %[[NV]] : ui256

// CHECK: sol.func @{{.*qualified.*}}() -> ui256 attributes {{.*}}selector = -228858638 : i32
// CHECK:   %[[QA:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[QC:.*]] = sol.call @{{.*halve.*}}(%[[QA]]) : (ui256) -> ui256
// CHECK:   sol.return %[[QC]] : ui256

// CHECK: sol.func @{{.*wrapped.*}}() -> ui256 attributes {{.*}}selector = 1357319496 : i32
// CHECK:   %[[WK:.*]] = sol.constant 6 : ui8
// CHECK:   %[[WV:.*]] = sol.cast %[[WK]] : ui8 to ui256
// CHECK:   sol.return %[[WV]] : ui256

import "./module_members.sol" as M;
import {FREE_K as RENAMED_K, freeTriple} from "./module_members.sol";
import * as S from "./module_members.sol";

uint256 constant FREE_K = 11;

type Cost is uint256;

function freeTriple(uint256 x) pure returns (uint256) {
    return x * 3;
}

contract ModuleMembers {
    function chained() public pure returns (uint256) {
        return M.M.M.FREE_K;
    }

    function renamed() public pure returns (uint256) {
        return freeTriple(RENAMED_K);
    }

    function starred() public pure returns (uint256) {
        return S.FREE_K;
    }

    function plainCall() public pure returns (uint256) {
        return M.freeTriple(4);
    }

    function chainedCall() public pure returns (uint256) {
        return M.M.freeTriple(4);
    }

    function starredCall() public pure returns (uint256) {
        return S.freeTriple(4);
    }

    function parenthesized() public pure returns (uint256) {
        return (M).FREE_K;
    }

    function qualified() public pure returns (uint256) {
        return M.ModuleMembers.halve(6);
    }

    function wrapped() public pure returns (uint256) {
        return Cost.unwrap(M.Cost.wrap(6));
    }

    function halve(uint256 x) internal pure returns (uint256) {
        return x / 2;
    }
}
