//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "dirtyS1Byte()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "getS1()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setS1(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "getS1()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dirtyS1Byte()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "setS1(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "getS1()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "initS2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "dirtyS2Byte5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "getS2_5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setS2_5(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "getS2_5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dirtyS2Byte5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "setS2_5(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "getS2_5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dirtyS3Byte5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "getS3_5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setS3_5(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "getS3_5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dirtyS3Byte5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "setS3_5(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "getS3_5()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint8 s0;
  bool s1;
  bool[] s2;
  bool[40] s3;

  function dirtyS1Byte() public {
    assembly {
      let slot := s1.slot
      let off := s1.offset
      let cur := sload(slot)
      let dirty := shl(mul(off, 8), 0x80)
      sstore(slot, or(cur, dirty))
    }
  }

  function setS1(bool v) public {
    s1 = v;
  }

  function getS1() public view returns (bool) {
    return s1;
  }

  function initS2() public {
    s2.push();
    s2.push();
    s2.push();
    s2.push();
    s2.push();
    s2.push();
  }

  function dirtyS2Byte5() public {
    assembly {
      let arrSlot := s2.slot
      mstore(0x00, arrSlot)
      let base := keccak256(0x00, 0x20)
      let slot := add(base, div(5, 32))
      let off := mod(5, 32)
      let cur := sload(slot)
      let dirty := shl(mul(off, 8), 0x80)
      sstore(slot, or(cur, dirty))
    }
  }

  function setS2_5(bool v) public {
    s2[5] = v;
  }

  function getS2_5() public view returns (bool) {
    return s2[5];
  }

  function dirtyS3Byte5() public {
    assembly {
      let base := s3.slot
      let slot := add(base, div(5, 32))
      let off := mod(5, 32)
      let cur := sload(slot)
      let dirty := shl(mul(off, 8), 0x80)
      sstore(slot, or(cur, dirty))
    }
  }

  function setS3_5(bool v) public {
    s3[5] = v;
  }

  function getS3_5() public view returns (bool) {
    return s3[5];
  }
}
