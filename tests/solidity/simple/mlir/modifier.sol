//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "20"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint m;
  modifier m0(uint a) {
    require(a == 1);
    _;
    a += 1;
    _;
  }

  modifier m1(uint a) {
    require(a == 1);
    m += 10;
    _;
  }

  function f(uint a) m0(a) m1(a) public returns (uint) {
    require(a == 1);
    return m;
  }
}
