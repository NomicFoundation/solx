//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "staticArrayRoundTrip(bytes4,bytes31)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000",
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ],
//!                     "expected": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000",
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dynamicArrayRoundTrip(bytes4,bytes31)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000",
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ],
//!                     "expected": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000",
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ]
//!                 },
//!                 {
//!                     "method": "structRoundTrip(bytes4,bytes31)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000",
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ],
//!                     "expected": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000",
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  struct Pair {
    bytes4 a;
    bytes31 b;
  }

  function staticArrayRoundTrip(bytes4 a, bytes31 b) public pure returns (bytes4, bytes31) {
    bytes4[1] memory arr4;
    bytes31[1] memory arr31;
    arr4[0] = a;
    arr31[0] = b;
    return (arr4[0], arr31[0]);
  }

  function dynamicArrayRoundTrip(bytes4 a, bytes31 b) public pure returns (bytes4, bytes31) {
    bytes4[] memory arr4 = new bytes4[](1);
    bytes31[] memory arr31 = new bytes31[](1);
    arr4[0] = a;
    arr31[0] = b;
    return (arr4[0], arr31[0]);
  }

  function structRoundTrip(bytes4 a, bytes31 b) public pure returns (bytes4, bytes31) {
    Pair memory p;
    p.a = a;
    p.b = b;
    return (p.a, p.b);
  }
}
