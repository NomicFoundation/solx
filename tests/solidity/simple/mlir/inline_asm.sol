//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "nonPackable()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "packable()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "1",
//!                         "1",
//!                         "0",
//!                         "1",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "localStoragePtr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2",
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
  function f() public pure returns (uint) {
    uint ret = 0;
    assembly {
      ret := 1
    }
    return ret;
  }

  // State variables for .slot/.offset testing
  uint256 x;           // slot 0, offset 0 (non-packable)
  uint8 a;             // slot 1, offset 0 (packable)
  uint8 b;             // slot 1, offset 1 (packable)
  uint16 c;            // slot 1, offset 2 (packable)
  uint256[] arr;       // slot 2, offset 0 (non-packable)

  // Test non-packable state vars (x, arr)
  function nonPackable() public pure returns (uint, uint, uint) {
    uint sx; uint ox; uint sarr;
    assembly {
      sx := x.slot
      ox := x.offset
      sarr := arr.slot
    }
    return (sx, ox, sarr);
  }

  // Test packable state vars (a, b, c)
  function packable() public pure returns (uint, uint, uint, uint, uint, uint) {
    uint sa; uint sb; uint sc; uint oa; uint ob; uint oc;
    assembly {
      sa := a.slot
      sb := b.slot
      sc := c.slot
      oa := a.offset
      ob := b.offset
      oc := c.offset
    }
    return (sa, sb, sc, oa, ob, oc);
  }

  // Test local storage pointer
  function getStoragePtr(uint256[] storage ref) internal pure returns (uint, uint) {
    uint s; uint o;
    assembly {
      s := ref.slot
      o := ref.offset
    }
    return (s, o);
  }

  function localStoragePtr() public view returns (uint, uint) {
    return getStoragePtr(arr);
  }
}
