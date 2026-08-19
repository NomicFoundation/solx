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
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_positive",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42", "3"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "126"
//!     ]
//! }, {
//!     "name": "ordinary_negative_first",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-42", "3"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-126"
//!     ]
//! }, {
//!     "name": "ordinary_negative_second",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42", "-3"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-126"
//!     ]
//! }, {
//!     "name": "ordinary_negative_both",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-42", "-3"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "126"
//!     ]
//! }, {
//!     "name": "to_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "170141183460469231731687303715884105727", "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "170141183460469231731687303715884105727"
//!     ]
//! }, {
//!     "name": "overflow_negative",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-110141183460469231731687303715884105727", "2"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [
//!             "120000000000000000000000000000000000002"
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
//!     "name": "overflow_positive",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "110141183460469231731687303715884105727", "2"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [
//!             "-120000000000000000000000000000000000002"
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
//!                 "21267647932558653966460912964485513216", "8"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<0.8",
//!         "return_data": [
//!             "-170141183460469231731687303715884105728"
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
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(int128 a, int128 b) public pure returns(int128) {
        int128 c = a;
        c *= b;
        return c;
    }
}
