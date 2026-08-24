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
//!                         "3"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint immutable i;
  uint immutable j;
  constructor() { i = 1; j = 2; }
  function f() public returns (uint) {
    uint ret;
    if (i == 2) // > 1 use of an immutable
      ret = i - j;
    else
      ret = i + j;
    return ret;
  }
}
