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
//!                     "method": "copy()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "readS2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "2",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "mutateS2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "readS1()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "2",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "readS2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "99",
//!                         "98",
//!                         "97"
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
  S s1;
  S s2;

  function build() public {
    s1.a = 1;
    s1.b.push();
    s1.b[0].a = 2;
    s1.b[0].b.push();
    s1.b[0].b[0].a = 3;
  }

  function copy() public {
    s2 = s1;
  }

  function readS2() public view returns (uint256, uint256, uint256) {
    return (s2.a, s2.b[0].a, s2.b[0].b[0].a);
  }

  function mutateS2() public {
    s2.a = 99;
    s2.b[0].a = 98;
    s2.b[0].b[0].a = 97;
  }

  function readS1() public view returns (uint256, uint256, uint256) {
    return (s1.a, s1.b[0].a, s1.b[0].b[0].a);
  }
}
