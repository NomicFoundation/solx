//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "computeA()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "computeB()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "11"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

function add(uint a, uint b) pure returns (uint) {
    return a + b;
}

contract Test {
    function computeA() public pure returns (uint) {
        return add(3, 4);
    }

    function computeB() public pure returns (uint) {
        return add(5, 6);
    }
}
