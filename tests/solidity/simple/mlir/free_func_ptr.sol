//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "computeViaPtr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
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
    function computeViaPtr() public pure returns (uint) {
        function (uint, uint) pure returns (uint) fp = add;
        return fp(3, 4);
    }
}
