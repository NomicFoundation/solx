//! { "cases": [ {
//!     "name": "main",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "$BLOB_HASH:0"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.24;

contract Test {
    function main() public view returns(bytes32) {
        bytes32 _blobhash = blobhash(0);
        return _blobhash;
    }
}
