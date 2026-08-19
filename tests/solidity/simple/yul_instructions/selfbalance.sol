//! { "cases": [ {
//!     "name": "zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!             ],
//!             "value": "0 wei"
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!             ],
//!             "value": "1 wei"
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!             ],
//!             "value": "433478394034343 wei"
//!         }
//!     ],
//!     "expected": [
//!         "433478394034343"
//!     ]
//! }, {
//!     "name": "2_in_128_minus_one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!             ],
//!             "value": "340282366920938463463374607431768211455 wei"
//!         }
//!     ],
//!     "expected": [
//!         "0xffffffffffffffffffffffffffffffff"
//!     ]
//! }, {
//!     "name": "2_in_128",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [],
//!             "value": "340282366920938463463374607431768211455 wei",
//!             "expected": [
//!                 "0xffffffffffffffffffffffffffffffff"
//!             ]
//!         }, {
//!             "method": "main",
//!             "calldata": [
//!             ],
//!             "value": "1 wei"
//!         }
//!     ],
//!     "expected": [
//!         "0x100000000000000000000000000000000"
//!     ]
//! }, {
//!     "name": "ordinary_bigger_2_in_128",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [],
//!             "value": "340282366920938463463374607431768211455 wei",
//!             "expected": [
//!                 "0xffffffffffffffffffffffffffffffff"
//!             ]
//!         }, {
//!             "method": "main",
//!             "calldata": [
//!             ],
//!             "value": "48384899813922913922990329232 wei"
//!         }
//!     ],
//!     "expected": [
//!         "0x1000000009C570D12AFFBBF971E8DCD8F"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.5.14;

contract Test {
    function main() external payable returns(uint256 result) {
        assembly {
            result := selfbalance()
        }
    }
}
