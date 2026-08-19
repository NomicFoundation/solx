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
//!     "name": "ordinary_true_lesser",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-25", "42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary_false_bigger",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42", "-25"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_false_negative",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-42", "-42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_false_positive",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42", "42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "min_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-170141183460469231731687303715884105728", "-170141183460469231731687303715884105728"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "170141183460469231731687303715884105727", "170141183460469231731687303715884105727"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(int128 a, int128 b) public pure returns(bool) {
        return a < b;
    }
}
