//! { "cases": [ {
//!     "name": "f",
//!     "inputs": [
//!         {
//!             "method": "f",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "7"
//!     ]
//! }, {
//!     "name": "g",
//!     "inputs": [
//!         {
//!             "method": "g",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "65"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function f() external pure returns (uint256) {
        return 7;
    }

    function g() external pure returns (uint256) {
        bytes memory b = hex"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        return b.length + uint8(b[32]);
    }
}
