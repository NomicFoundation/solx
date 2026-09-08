//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "m(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "0",
//!                         "0x40",
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0xC0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setX(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setInnerA(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setInnerB(uint256,bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "42",
//!                         "0x40",
//!                         "7",
//!                         "1",
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0xC0",
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// Verifies that the auto-generated public getter for a mapping-to-struct
// correctly handles nested structs containing arrays.
// The array-exclusion filter in FunctionType(VariableDeclaration) only applies
// to direct members of the top-level struct (Outer). Inner is returned as a whole
// StructType<Memory>, so its array members (staticArr, dynArr) ARE included in
// the ABI return type.
// Return type: (uint256, (uint256, bool, uint256[3], uint256[]))
//   Wire layout: x, 0x40, a, b, arr[0], arr[1], arr[2], 0xC0, dynLen
contract Test {
  struct Inner {
    uint256 a;
    bool b;
    uint256[3] staticArr;
    uint256[] dynArr;
  }
  struct Outer {
    uint256 x;
    Inner inner;
  }

  mapping(uint256 => Outer) public m;

  function setX(uint256 key, uint256 val) public { m[key].x = val; }
  function setInnerA(uint256 key, uint256 val) public { m[key].inner.a = val; }
  function setInnerB(uint256 key, bool val) public { m[key].inner.b = val; }
}
