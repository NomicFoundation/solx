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
//!                 },
//!                 {
//!                     "method": "n()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "sum()",
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
  uint public n;
  uint public sum;

  modifier twice() {
    _;
    _;
  }

  modifier add(uint v) {
    sum += v;
    _;
  }

  function g() internal returns (uint) {
    n += 1;
    return n;
  }

  function f() public twice add(g()) returns (uint) {
    return sum;
  }
}
