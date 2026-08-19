//! { "cases": [ {
//!     "name": "zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "42"
//!     ]
//! }, {
//!     "name": "max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "127"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "127"
//!     ]
//! }, {
//!     "name": "overflow_negative_minimal",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "255"
//!     ]
//! }, {
//!     "name": "overflow_negative",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "214"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(int8 a) public pure returns(uint8) {
        return uint8(a);
    }
}
