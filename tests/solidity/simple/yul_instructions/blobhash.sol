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
//!         "$BLOB_HASH:0"
//!     ]
//! }, {
//!     "name": "one",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "1"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "$BLOB_HASH:1"
//!     ]
//! }, {
//!     "name": "ordinar",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "5"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.24;

contract Test {
    function main(uint256 a) external view returns(uint256 result) {
        assembly {
            result := blobhash(a)
        }
    }
}
