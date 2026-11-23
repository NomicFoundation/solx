//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "assigned",
//!             "inputs": [
//!                 {
//!                     "method": "assigned()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "16"
//!                     ]
//!                 }
//!             ]
//!         },
//!         {
//!             "name": "unassigned",
//!             "inputs": [
//!                 {
//!                     "method": "unassigned()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "102"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 counter;

    modifier twice() {
        _;
        _;
    }

    function assigned() external twice returns (uint256 r) {
        if (counter++ == 0) {
            r = 5;
            return r + 1;
        }
        r += 10;
    }

    function unassigned() external twice returns (uint256 r) {
        if (r == 0) {
            return 100;
        }
        r += 2;
    }
}
