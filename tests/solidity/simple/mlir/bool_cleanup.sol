//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "mem_dirty_bool()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_storage_dirty_bool1()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "read_storage_dirty_bool()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_storage_dirty_bool2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "read_storage_dirty_bool()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_storage_dirty_bool_array()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "read_storage_dirty_bool_array()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ep_bool_array_dirty_memory()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "32",
//!                         "32",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ei_bool_array_dirty_memory()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "32",
//!                         "96",
//!                         "32",
//!                         "1",
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
  bool s0;
  bool[] s1;

  function mem_dirty_bool() public returns (bool) {
    bool[] memory a = new bool[](1);
    assembly {
      mstore(add(a, 0x20), 2)
    }
    return a[0];
  }

  function set_storage_dirty_bool1() public {
    assembly {
      sstore(s0.slot, 2)
    }
  }

  function set_storage_dirty_bool2() public {
    assembly {
      sstore(s0.slot, 256)
    }
  }

  function read_storage_dirty_bool() public view returns (bool) {
    return s0;
  }

  function set_storage_dirty_bool_array() public {
    s1.push();
    assembly {
      mstore(0x00, s1.slot)
      let base := keccak256(0x00, 0x20)
      sstore(base, 2)
    }
  }

  function read_storage_dirty_bool_array() public view returns (bool) {
    return s1[0];
  }

  function ep_bool_array_dirty_memory() public returns (bytes memory) {
    bool[] memory a = new bool[](1);
    assembly {
      mstore(add(a, 0x20), 2)
    }
    return abi.encodePacked(a);
  }

  function ei_bool_array_dirty_memory() public returns (bytes memory) {
    bool[] memory a = new bool[](1);
    assembly {
      mstore(add(a, 0x20), 2)
    }
    return abi.encode(a);
  }
}
