//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "newStructArray()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "newStructArrayNoAlias()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "newNestedDyn()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "structArrayMember()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fixedMultiDim()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "bareDynDefault()",
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

pragma solidity >=0.8.0;

contract Test {
  struct S {
    uint256 x;
    string s;
    uint256[] a;
  }

  // Pollute keccak scratch space so that any element/member slot left at 0
  // (instead of the 0x60 sentinel) reads back garbage through mload(0).
  function dirtyScratch() internal pure {
    assembly {
      mstore(0, 77)
    }
  }

  // new S[](n): every element is a distinct allocation with default members.
  function newStructArray() public pure returns (uint256, uint256, uint256) {
    dirtyScratch();
    S[] memory arr = new S[](2);
    return (arr[1].x, bytes(arr[1].s).length, arr[1].a.length);
  }

  // Elements must not alias: writing one element's member leaves the others
  // at their defaults.
  function newStructArrayNoAlias() public pure returns (uint256, uint256) {
    S[] memory arr = new S[](2);
    arr[0].x = 42;
    return (arr[0].x, arr[1].x);
  }

  // new uint[][](n): inner elements default to empty arrays.
  function newNestedDyn() public pure returns (uint256, uint256) {
    dirtyScratch();
    uint256[][] memory a = new uint256[][](3);
    return (a[0].length, a[2].length);
  }

  // Default-initialized struct with a dynamic array member.
  function structArrayMember() public pure returns (uint256, uint256) {
    dirtyScratch();
    S memory s;
    return (s.x, s.a.length);
  }

  // Fixed multi-dimensional array: the whole payload must be zeroed even
  // when the free memory was dirtied beforehand.
  function fixedMultiDim() public pure returns (uint256, uint256) {
    assembly {
      let p := mload(0x40)
      mstore(p, 11)
      mstore(add(p, 0x40), 22)
      mstore(add(p, 0xa0), 33)
    }
    uint256[2][3] memory x;
    return (x[0][0], x[2][1]);
  }

  // Bare defaults of dynamically sized locals: the zero-pointer sentinel.
  function bareDynDefault() public pure returns (uint256, uint256) {
    dirtyScratch();
    uint256[] memory u;
    string memory s;
    return (u.length, bytes(s).length);
  }
}
