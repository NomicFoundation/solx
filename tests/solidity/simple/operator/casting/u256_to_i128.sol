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
//!                 "170141183460469231731687303715884105727"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "170141183460469231731687303715884105727"
//!     ]
//! }, {
//!     "name": "overflow_positive_minimal",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "170141183460469231731687303715884105728"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-170141183460469231731687303715884105728"
//!     ]
//! }, {
//!     "name": "overflow_positive",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "21270141183460469231731687303715884105728"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-167647932558653966460912964485513216000"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(uint256 a) public pure returns(int128) {
        return int128(int256(a));
    }
}
