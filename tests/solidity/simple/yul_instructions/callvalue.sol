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
//!     "name": "max",
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
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main() external payable returns(uint256 result) {
        assembly {
            result := callvalue()
        }
    }
}
