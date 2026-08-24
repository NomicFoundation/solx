//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function f() public pure returns (bool correct) {
        uint8[1] memory m;
        assembly {
            mstore(m, 257)
        }
        uint8 x = m[0];
        uint r;
        assembly {
            r := x
        }
        correct = (m[0] == 0x01) && (r == 0x01);
    }
}
