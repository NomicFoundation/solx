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
//!                     "method": "copySto()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "readS2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "100",
//!                         "1000",
//!                         "101",
//!                         "1001"
//!                     ]
//!                 },
//!                 {
//!                     "method": "copyMem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "100",
//!                         "1000",
//!                         "101",
//!                         "1001"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

struct S {
  uint16 v;
  uint32 w;
  S[] x;
}

contract Test {
  S s1;
  S s2;

  function build() public {
    s1.v = 100;
    s1.w = 1000;
    s1.x.push();
    s1.x[0].v = 101;
    s1.x[0].w = 1001;
  }

  function copySto() public {
    s2 = s1;
  }

  function readS2() public view returns (uint16, uint32, uint16, uint32) {
    return (s2.v, s2.w, s2.x[0].v, s2.x[0].w);
  }

  function copyMem() public view returns (uint16, uint32, uint16, uint32) {
    S memory m = s1;
    return (m.v, m.w, m.x[0].v, m.x[0].w);
  }
}
