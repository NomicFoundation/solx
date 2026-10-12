//! { "cases": [ {
//!     "name": "scalar",
//!     "inputs": [ { "method": "scalar", "calldata": [] } ],
//!     "expected": [ "0x000000000000000000000000000000ffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "from_memory",
//!     "inputs": [ { "method": "fromMemory", "calldata": [] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "from_calldata",
//!     "inputs": [ { "method": "fromCalldata", "calldata": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "from_storage",
//!     "inputs": [ { "method": "fromStorage", "calldata": [] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "same_type_memory",
//!     "inputs": [ { "method": "sameTypeMemory", "calldata": [] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "same_type_calldata",
//!     "inputs": [ { "method": "sameTypeCalldata", "calldata": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// An array copy into storage writes an element owning its slot as the whole
// cleaned word, while a scalar store masks it to the type's width.
contract Test {
  int136 s;
  int136[1] a;
  int8[1] p;

  function slotA() internal view returns (bytes32 r) {
    assembly { r := sload(a.slot) }
  }

  function scalar() external returns (bytes32 r) {
    s = -1;
    assembly { r := sload(s.slot) }
  }

  function fromMemory() external returns (bytes32) {
    int8[1] memory m = [int8(-1)];
    a = m;
    return slotA();
  }

  function fromCalldata(int8[1] calldata c) external returns (bytes32) {
    a = c;
    return slotA();
  }

  function fromStorage() external returns (bytes32) {
    p[0] = -1;
    a = p;
    return slotA();
  }

  function sameTypeMemory() external returns (bytes32) {
    int136[1] memory m = [int136(-1)];
    a = m;
    return slotA();
  }

  function sameTypeCalldata(int136[1] calldata c) external returns (bytes32) {
    a = c;
    return slotA();
  }
}
