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
//!                     "method": "copyMemLens()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "4",
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "rootLen()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "del()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "rootLen()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
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
  string name;
  S[] kids;
}

contract Test {
  S s;

  function build() public {
    s.name = "root";
    s.kids.push();
    s.kids[0].name = "child";
  }

  function copyMemLens() public view returns (uint256, uint256) {
    S memory m = s;
    return (bytes(m.name).length, bytes(m.kids[0].name).length);
  }

  function del() public {
    delete s;
  }

  function rootLen() public view returns (uint256) {
    return bytes(s.name).length;
  }
}
