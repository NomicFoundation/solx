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
//!         "0", "0"
//!     ]
//! }, {
//!     "name": "ordinary_true_lesser",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "25", "42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "25", "42"
//!     ]
//! }, {
//!     "name": "ordinary_true_bigger",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42", "25"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "25", "42"
//!     ]
//! }, {
//!     "name": "ordinary_false",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42", "42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "42", "42"
//!     ]
//! }, {
//!     "name": "max_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffff"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffff"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(address tokenA, address tokenB) public pure returns (address token0, address token1) {
        (token0, token1) = tokenA < tokenB ? (tokenA, tokenB) : (tokenB, tokenA);
    }
}
