//! {
//!     "modes": [
//!         "Y"
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
//!                     "method": "read()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "2",
//!                         "2",
//!                         "3",
//!                         "1",
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

  function build() public {
    S memory m;
    m.a = 1;
    m.b = new S[](2);
    m.b[0].a = 2;
    m.b[1].a = 3;
    m.b[0].b = new S[](1);
    m.b[0].b[0].a = 4;
    s = m;
  }

  function read()
    public
    view
    returns (uint256, uint256, uint256, uint256, uint256, uint256)
  {
    return (
      s.a,
      s.b.length,
      s.b[0].a,
      s.b[1].a,
      s.b[0].b.length,
      s.b[0].b[0].a
    );
  }
}
