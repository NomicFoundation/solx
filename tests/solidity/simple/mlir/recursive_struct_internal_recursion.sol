//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "total()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "10"
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

  constructor() {
    s.a = 1;
    s.b.push();
    s.b[0].a = 2;
    s.b.push();
    s.b[1].a = 3;
    s.b[0].b.push();
    s.b[0].b[0].a = 4;
  }

  function sum(S memory t) internal pure returns (uint256 acc) {
    acc = t.a;
    for (uint256 i = 0; i < t.b.length; i++)
      acc += sum(t.b[i]);
  }

  function total() public view returns (uint256) {
    S memory m = s;
    return sum(m);
  }
}
