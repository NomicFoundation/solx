//! {
//!     "modes": [
//!         "Y"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "pushMem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "read()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2",
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "2"
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

  function pushMem() public {
    S memory m;
    m.a = 1;
    m.b = new S[](1);
    m.b[0].a = 2;
    s.b.push(m);
    m.a = 3;
    s.b.push(m);
  }

  function read()
    public
    view
    returns (uint256, uint256, uint256, uint256, uint256)
  {
    return (s.b.length, s.b[0].a, s.b[0].b[0].a, s.b[1].a, s.b[1].b[0].a);
  }
}
