//! { "cases": [ {
//!     "name": "calldata_uint",
//!     "inputs": [ { "method": "copyUint", "calldata": [ "0x20", "1", "256" ] } ],
//!     "expected": { "return_data": [], "exception": true }
//! }, {
//!     "name": "calldata_int",
//!     "inputs": [ { "method": "copyInt", "calldata": [ "0x20", "1", "128" ] } ],
//!     "expected": { "return_data": [], "exception": true }
//! }, {
//!     "name": "calldata_bytes",
//!     "inputs": [ { "method": "copyBytes", "calldata": [ "0x20", "1", "0x100000000000000000000000000000000" ] } ],
//!     "expected": { "return_data": [], "exception": true }
//! }, {
//!     "name": "memory_int",
//!     "inputs": [ { "method": "copyDirtyMemory", "calldata": [] } ],
//!     "expected": [ "-128" ]
//! }, {
//!     "name": "storage_int",
//!     "inputs": [ { "method": "copyStorage", "calldata": [] } ],
//!     "expected": [ "-1" ]
//! }, {
//!     "name": "storage_packed_int",
//!     "inputs": [ { "method": "copyPackedStorage", "calldata": [] } ],
//!     "expected": [ "-1" ]
//! }, {
//!     "name": "storage_packed_to_unpacked_int",
//!     "inputs": [ { "method": "copyPackedStorageToUnpacked", "calldata": [] } ],
//!     "expected": [ "-1" ]
//! }, {
//!     "name": "storage_bytes",
//!     "inputs": [ { "method": "copyStorageBytes", "calldata": [] } ],
//!     "expected": [ "0x1000000000000000000000000" ]
//! }, {
//!     "name": "memory_bool",
//!     "inputs": [ { "method": "copyDirtyMemoryBool", "calldata": [] } ],
//!     "expected": [ "1" ]
//! }, {
//!     "name": "memory_unpacked_int",
//!     "inputs": [ { "method": "copyMemoryUnpacked", "calldata": [] } ],
//!     "expected": [ "-2" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// An array copy into storage reads the source element as its own type: a dirty
// calldata element reverts, a dirty memory element is canonicalized, a storage
// element is sign-extended or left-aligned before it is widened, and an element
// alone in its slot is stored as its whole canonical word.
contract Test {
  uint16[] s16;
  int8[] si8;
  int16[] si16;
  bytes16[] sb16;
  int136[] si136;
  int256[] si256;
  bytes20[] sb20;
  bytes32[] sb32;
  bool[] sb;

  function copyUint(uint8[] calldata a) external {
    s16 = a;
  }

  function copyInt(int8[] calldata a) external {
    si16 = a;
  }

  function copyBytes(bytes15[] calldata a) external {
    sb16 = a;
  }

  function copyDirtyMemory() external returns (int16) {
    int8[] memory a = new int8[](1);
    // -128 as int8
    assembly { mstore(add(a, 32), 0x80) }
    si16 = a;
    return si16[0];
  }

  function copyStorage() external returns (int256) {
    si136.push(-1);
    si256 = si136;
    return si256[0];
  }

  function copyPackedStorage() external returns (int16) {
    si8.push(-1);
    si16 = si8;
    return si16[0];
  }

  function copyPackedStorageToUnpacked() external returns (int136) {
    si8.push(-1);
    si136 = si8;
    return si136[0];
  }

  function copyStorageBytes() external returns (bytes32) {
    sb20.push(bytes20(uint160(1)));
    sb32 = sb20;
    return sb32[0];
  }

  function copyDirtyMemoryBool() external returns (bool) {
    bool[] memory a = new bool[](1);
    assembly { mstore(add(a, 32), 0x100) }
    sb = a;
    return sb[0];
  }

  function copyMemoryUnpacked() external returns (bytes32 r) {
    int136[] memory a = new int136[](1);
    a[0] = -2;
    si136 = a;
    assembly {
      mstore(0, si136.slot)
      r := sload(keccak256(0, 32))
    }
  }
}
