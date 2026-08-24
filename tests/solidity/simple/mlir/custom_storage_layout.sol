//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "slots()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "100",
//!                         "101",
//!                         "101",
//!                         "102"
//!                     ]
//!                 },
//!                 {
//!                     "method": "offsets()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

uint256 constant BASE = 50;

contract A {
  uint256 a;  // slot 100 (inherited)
}

contract Test is A layout at BASE * 2 {
  uint8 b;    // slot 101, offset 0
  uint8 c;    // slot 101, offset 1
  uint256 d;  // slot 102

  function slots() public pure returns (uint, uint, uint, uint) {
    uint sa; uint sb; uint sc; uint sd;
    assembly {
      sa := a.slot
      sb := b.slot
      sc := c.slot
      sd := d.slot
    }
    return (sa, sb, sc, sd);
  }

  function offsets() public pure returns (uint, uint) {
    uint ob; uint oc;
    assembly {
      ob := b.offset
      oc := c.offset
    }
    return (ob, oc);
  }
}
