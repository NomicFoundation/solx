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
//!         "1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.24;

contract Test {
    function main() public view returns(uint256) {
        uint256 _blobbasefee = block.blobbasefee;
        return _blobbasefee;
    }
}
