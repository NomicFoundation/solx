//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "test()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

struct S {
  uint256 a;
  S[] b;
}

contract Test {
  S s;

  function test() public returns (uint256, uint256) {
    s.b.push();
    s.b[0].a = 5;
    s.b[0].b.push();
    s.b[0].b[0].a = 6;
    s.b.pop();
    s.b.push();
    return (s.b[0].a, s.b[0].b.length);
  }
}
