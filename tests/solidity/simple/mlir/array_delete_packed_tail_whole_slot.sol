//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "member()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "topLevel()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "twoSlots()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "sevenSlots()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    struct S { uint8[31] arr; }
    S s;
    uint8[31] top;
    uint8[40] two;
    uint8[200] seven;

    // uint8[31] as a struct member: one slot, 31 bytes used, 1 unused.
    function member() public returns (uint256 r) {
        assembly { sstore(s.slot, not(0)) }
        s.arr[0] = 1; s.arr[30] = 2;
        delete s;
        assembly { r := sload(s.slot) }
    }

    // The same array as a top-level state variable.
    function topLevel() public returns (uint256 r) {
        assembly { sstore(top.slot, not(0)) }
        top[0] = 1; top[30] = 2;
        delete top;
        assembly { r := sload(top.slot) }
    }

    // Two slots: the second holds 8 elements, 24 bytes unused.
    function twoSlots() public returns (uint256 r0, uint256 r1) {
        assembly {
            sstore(two.slot, not(0))
            sstore(add(two.slot, 1), not(0))
        }
        two[0] = 1; two[39] = 2;
        delete two;
        assembly {
            r0 := sload(two.slot)
            r1 := sload(add(two.slot, 1))
        }
    }

    // Seven slots: above the five the old code generator unrolls, so its
    // clearStorageLoop(uint256) path runs; the last slot holds 8 elements.
    function sevenSlots() public returns (uint256 r6) {
        assembly { sstore(add(seven.slot, 6), not(0)) }
        seven[0] = 1; seven[199] = 2;
        delete seven;
        assembly { r6 := sload(add(seven.slot, 6)) }
    }
}
