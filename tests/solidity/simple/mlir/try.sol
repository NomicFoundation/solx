//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f(bool,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f(bool,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "0x11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f(bool,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "1"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x08c379a000000000000000000000000000000000000000000000000000000000",
//!                             "0x0000002000000000000000000000000000000000000000000000000000000000",
//!                             "0x00000006666f6f62617200000000000000000000000000000000000000000000",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000006",
//!                         "0x666f6f6261720000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  function e(bool a, uint b) public returns (uint) {
    require(a, "foobar");
    return 2 - b;
  }

  function f(bool a, uint b) public returns (uint) {
    uint r = 0;
    try this.e(a, b) returns (uint ret) {
      r += ret;
    } catch Panic (uint code) {
      r += code;
    }

    return r;
  }

  function g() public returns (string memory) {
    string memory r;

    try this.e(false, 0) {
    } catch Error (string memory message) {
      r = message;
    }

    return r;
  }
}
