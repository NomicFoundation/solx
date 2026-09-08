//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "m(uint256,uint256)",
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
//!                     "method": "m(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "n(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0x00",
//!                         "0x00"
//!                     ]
//!                 },
//!                 {
//!                     "method": "n(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "7",
//!                         "8"
//!                     ]
//!                 },
//!                 {
//!                     "method": "n(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "9",
//!                         "0x0a"
//!                     ]
//!                 },
//!                 {
//!                     "method": "n(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "0x00",
//!                         "0x00"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// Verifies that the auto-generated public getter for a mapping-to-array-of-struct
// correctly resolves the mapping key (MapOp) and then indexes the inner array
// (GepOp) with a bounds check. Covers both a dynamic array (m) and a fixed-size
// array (n).
contract Test {
    struct Y {
        uint a;
        uint b;
    }
    mapping(uint256 => Y[]) public m;
    mapping(uint256 => Y[3]) public n;
    constructor() {
        m[1].push().a = 1;
        m[1][0].b = 2;
        m[1].push().a = 3;
        m[1][1].b = 4;
        n[1][0].a = 7;
        n[1][0].b = 8;
        n[1][1].a = 9;
        n[1][1].b = 10;
    }
}
