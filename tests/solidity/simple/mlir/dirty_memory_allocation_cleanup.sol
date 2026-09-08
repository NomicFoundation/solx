//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "dirtyNewArrayLength(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x100000000000000000000000000000000000000000000000000000000000001"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dirtyArrayLiteralElement(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x100000000000000000000000000000000000000000000000000000000000001"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function dirtyNewArrayLength(uint256 dirty) public pure returns (uint256) {
        uint8 n;
        assembly { n := dirty }

        uint256[] memory a = new uint256[](n);
        return a.length;
    }

    function dirtyArrayLiteralElement(uint256 dirty) public pure returns (uint256 r) {
        uint8 n;
        assembly { n := dirty }

        uint8[1] memory a = [n];
        assembly { r := mload(a) }
    }
}
