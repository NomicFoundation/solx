//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "a(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "a(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "a(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "a(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2",
//!                         "0"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// Verifies the getter for a fixed-outer / dynamic-inner packed array
// (uint8[][2]): outer bounds check, inner bounds check, packed uint8 slot
// extraction (keccak of inner-array slot, index/32, byte shift & AND 0xff).
contract Test {
    uint8[][2] public a;
    constructor() {
        a[1].push(3);
        a[1].push(4);
    }
}
