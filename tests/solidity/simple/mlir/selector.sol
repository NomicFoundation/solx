//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "selector()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x26121ff000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "sideEffect()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint x;

  function f() public {}

  function get() public returns (Test) {
    x = 42;
    return this;
  }

  function selector() public returns (bytes4) {
    return this.f.selector;
  }

  function sideEffect() public returns (uint) {
    get().f.selector;
    return x;
  }
}
