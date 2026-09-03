//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "build()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copyToMem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4"
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

  // Builds a 3-level tree:
  //   s              (a = 1)
  //   ├─ s.b[0]      (a = 2)
  //   │  └─ b[0].b[0] (a = 3)
  //   └─ s.b[1]      (a = 4)
  function build() public {
    s.a = 1;
    s.b.push();
    s.b[0].a = 2;
    s.b[0].b.push();
    s.b[0].b[0].a = 3;
    s.b.push();
    s.b[1].a = 4;
  }

  function copyToMem()
    public
    view
    returns (uint256, uint256, uint256, uint256)
  {
    S memory m = s;
    return (m.a, m.b[0].a, m.b[0].b[0].a, m.b[1].a);
  }
}
