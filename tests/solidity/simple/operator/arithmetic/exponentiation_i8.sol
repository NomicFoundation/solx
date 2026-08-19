//! { "cases": [ {
//!     "name": "zero_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary_positive",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-3", "4"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "81"
//!     ]
//! }, {
//!     "name": "ordinary_negative",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-4", "3"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-64"
//!     ]
//! }, {
//!     "name": "to_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "11", "2"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "121"
//!     ]
//! }, {
//!     "name": "overflow",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "100", "2"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [
//!             "16"
//!         ],
//!         "exception": false
//!     }, {
//!         "compiler_version": ">=0.8",
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     } ]
//! }, {
//!     "name": "overflow_minimal",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "12", "2"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [
//!             "115792089237316195423570985008687907853269984665640564039457584007913129639824"
//!         ],
//!         "exception": false
//!     }, {
//!         "compiler_version": ">=0.8",
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     } ]
//! }, {
//!     "name": "max_pow_overflow",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-2", "255"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [
//!             "0"
//!         ],
//!         "exception": false
//!     }, {
//!         "compiler_version": ">=0.8",
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     } ]
//! }, {
//!     "name": "0_max_pow",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0", "255"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "1_max_pow",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1", "255"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "1n_max_pow",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-1", "255"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.6.0;

contract Test {
    function main(int8 a, uint8 b) public pure returns(int8) {
        return a ** b;
    }
}
