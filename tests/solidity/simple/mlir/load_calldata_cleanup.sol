//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "loadCalldataByte(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                         "0x5800000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x5800000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadCalldataBool(bool[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadCalldataBool(bool[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "2"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "discardCalldataBool(bool[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "discardCalldataBool(bool[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "2"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "loadCalldataUint8(uint8[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x34"
//!                     ],
//!                     "expected": [
//!                         "0x34"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadCalldataUint8(uint8[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x1234"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "loadCalldataInt8(int8[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x7f"
//!                     ],
//!                     "expected": [
//!                         "127"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadCalldataInt8(int8[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x1234"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "loadCalldataEnum(uint8[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadCalldataEnum(uint8[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "3"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "loadCalldataAddress(address[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x1234123412341234123412341234123412341234"
//!                     ],
//!                     "expected": [
//!                         "0x1234123412341234123412341234123412341234"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadCalldataAddress(address[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x10000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "loadCalldataBytes4(bytes4[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadCalldataBytes4(bytes4[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x1234567800000000000000000000000000000000000000000000000000000001"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "loadCalldataBytes31(bytes31[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ],
//!                     "expected": [
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadCalldataBytes31(bytes31[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fff"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "loadCalldataFnPtrSuccess()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadCalldataFnPtrHelper(function[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "1",
//!                         "0x1234123412341234123412341234123412341234112233440000000000000001"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  enum E { A, B, Test }

  function add(uint a, uint b) external pure returns (uint) {
    return a + b;
  }

  function loadCalldataByte(bytes calldata data) external pure returns (bytes1) {
    return data[0];
  }

  function loadCalldataBool(bool[] calldata values) external pure returns (bool) {
    return values[0];
  }

  function discardCalldataBool(bool[] calldata values) external pure returns (uint8) {
    values[0];
    return 1;
  }

  function loadCalldataUint8(uint8[] calldata values) external pure returns (uint8) {
    return values[0];
  }

  function loadCalldataInt8(int8[] calldata values) external pure returns (int8) {
    return values[0];
  }

  function loadCalldataEnum(E[] calldata values) external pure returns (uint8) {
    return uint8(values[0]);
  }

  function loadCalldataAddress(address[] calldata values) external pure returns (address) {
    return values[0];
  }

  function loadCalldataBytes4(bytes4[] calldata values) external pure returns (bytes4) {
    return values[0];
  }

  function loadCalldataBytes31(bytes31[] calldata values) external pure returns (bytes31) {
    return values[0];
  }

  function loadCalldataFnPtrHelper(function(uint, uint) external pure returns (uint)[] calldata values) external view returns (uint) {
    return values[0](20, 22);
  }

  function loadCalldataFnPtrSuccess() external returns (uint) {
    function(uint, uint) external pure returns (uint)[] memory values =
      new function(uint, uint) external pure returns (uint)[](1);
    values[0] = this.add;
    return this.loadCalldataFnPtrHelper(values);
  }
}
