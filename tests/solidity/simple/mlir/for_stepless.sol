//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "infinite_break()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "no_step(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "7"
//!                     ],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function infinite_break() external pure returns (uint) {
        uint i = 0;
        for (;;) {
            i += 1;
            if (i == 5) {
                break;
            }
        }
        return i;
    }

    function no_step(uint n) external pure returns (uint) {
        uint i = 0;
        for (; i < n;) {
            i += 1;
        }
        return i;
    }
}
