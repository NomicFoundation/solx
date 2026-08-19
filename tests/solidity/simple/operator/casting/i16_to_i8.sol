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
//!                 "-129"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "127"
//!     ]
//! }, {
//!     "name": "overflow_negative",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-1000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "24"
//!     ]
//! }, {
//!     "name": "overflow_positive_minimal",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "128"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-128"
//!     ]
//! }, {
//!     "name": "overflow_positive",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1000"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-24"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(int16 a) public pure returns(int8) {
        return int8(a);
    }
}
