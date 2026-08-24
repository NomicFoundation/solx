//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "write(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "7",
//!                         "99"
//!                     ]
//!                 },
//!                 {
//!                     "method": "read(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "7"
//!                     ],
//!                     "expected": [
//!                         "99"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// Verifies that a named `S storage s` return parameter's .slot can be
// written from inline assembly.
contract Test {
    struct S {
        uint256 x;
    }

    function getS(uint256 targetSlot) internal pure returns (S storage s) {
        assembly {
            s.slot := targetSlot
        }
    }

    function write(uint256 targetSlot, uint256 val) public {
        getS(targetSlot).x = val;
    }

    function read(uint256 targetSlot) public view returns (uint256) {
        return getS(targetSlot).x;
    }
}
