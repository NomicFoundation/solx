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
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main() external returns(uint256) {
        assembly {
            selfdestruct(caller())
        }
        return 5;
    }
}
