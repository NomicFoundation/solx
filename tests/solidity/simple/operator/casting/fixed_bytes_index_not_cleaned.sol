//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0x1122334455667788990011223344556677889900112233445566778899001122", "256"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x1100000000000000000000000000000000000000000000000000000000000000"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function main(bytes32 value, uint256 index) public pure returns(bytes1) {
        return value[uint8(index)];
    }
}
