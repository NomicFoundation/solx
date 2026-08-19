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
//!     "name": "ordinary_negative",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "42"
//!     ]
//! }, {
//!     "name": "ordinary_positive",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-42"
//!     ]
//! }, {
//!     "name": "max_to_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "170141183460469231731687303715884105727"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-170141183460469231731687303715884105727"
//!     ]
//! }, {
//!     "name": "almost_min_to_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "-170141183460469231731687303715884105727"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "170141183460469231731687303715884105727"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(int128 a) public pure returns(int128) {
        return -a;
    }
}
