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
//!                     "method": "read()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "2",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "len()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "clear()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "len()",
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

struct A {
  uint256 av;
  B[] bst;
}

struct B {
  uint256 bv;
  A[] ast;
}


contract Test {
  A a;

  function build() public {
    a.av = 1;
    a.bst.push();
    a.bst[0].bv = 2;
    a.bst[0].ast.push();
    a.bst[0].ast[0].av = 3;
  }

  function read() public view returns (uint256, uint256, uint256) {
    return (a.av, a.bst[0].bv, a.bst[0].ast[0].av);
  }

  function clear() public {
    delete a;
  }

  function len() public view returns (uint256) {
    return a.bst.length;
  }
}
