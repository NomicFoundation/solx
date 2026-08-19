//! { "cases": [ {
//!     "name": "zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!             ],
//!             "caller": "0x0000000000000000000000000000000000000000"
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
//!             "caller": "0x0000000000000000000000000000000000000001"
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
//!             "caller": "0xA8740AF4B880AEE34245AC232323BB4A3424432F"
//!         }
//!     ],
//!     "expected": [
//!         "0xA8740AF4B880AEE34245AC232323BB4A3424432F"
//!     ]
//! }, {
//!     "name": "max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!             ],
//!             "caller": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!         }
//!     ],
//!     "expected": [
//!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main() external payable returns(uint256 result) {
        assembly {
            result := caller()
        }
    }
}
