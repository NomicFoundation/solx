//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "loadMemoryByte()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x5800000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeMemoryByte()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x5800000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadMemoryBool()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeMemoryBool(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadMemoryUint8()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x34"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeMemoryUint8(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x34"
//!                     ],
//!                     "expected": [
//!                         "0x34"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadMemoryInt8()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeMemoryInt8(int8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "-1"
//!                     ],
//!                     "expected": [
//!                         "-1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadMemoryEnum()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeMemoryEnum()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadMemoryEnumInvalid()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000002100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "storeMemoryEnumInvalid()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000002100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "discardMemoryEnumInvalid()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadMemoryAddress()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x1234123412341234123412341234123412341234"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeMemoryAddress()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x1234123412341234123412341234123412341234"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadMemoryBytes4()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeMemoryBytes4()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadMemoryBytes31()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeMemoryBytes31()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f00"
//!                     ]
//!                 },
//!                 {
//!                     "method": "loadMemoryFnPtr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storeMemoryFnPtr()",
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
  enum E { A, B, Test }

  function add(uint a, uint b) external pure returns (uint) {
    return a + b;
  }

  function loadMemoryByte() public pure returns (bytes1) {
    bytes memory data = new bytes(1);
    assembly {
      mstore(add(data, 0x20), shl(248, 0x58))
    }
    return data[0];
  }

  function storeMemoryByte() public pure returns (bytes1) {
    bytes memory data = new bytes(1);
    data[0] = "X";
    return data[0];
  }

  function loadMemoryBool() public pure returns (bool) {
    bool[] memory values = new bool[](1);
    assembly {
      mstore(add(values, 0x20), 2)
    }
    return values[0];
  }

  function storeMemoryBool(bool value) public pure returns (bool) {
    bool[] memory values = new bool[](1);
    values[0] = value;
    return values[0];
  }

  function loadMemoryUint8() public pure returns (uint8) {
    uint8[] memory values = new uint8[](1);
    assembly {
      mstore(add(values, 0x20), 0x1234)
    }
    return values[0];
  }

  function storeMemoryUint8(uint8 value) public pure returns (uint8) {
    uint8[] memory values = new uint8[](1);
    values[0] = value;
    return values[0];
  }

  function loadMemoryInt8() public pure returns (int8) {
    int8[] memory values = new int8[](1);
    assembly {
      mstore(add(values, 0x20), 0x1ff)
    }
    return values[0];
  }

  function storeMemoryInt8(int8 value) public pure returns (int8) {
    int8[] memory values = new int8[](1);
    values[0] = value;
    return values[0];
  }

  function loadMemoryEnum() public pure returns (uint8) {
    E[] memory values = new E[](1);
    assembly {
      mstore(add(values, 0x20), 2)
    }
    return uint8(values[0]);
  }

  function storeMemoryEnum() public pure returns (uint8) {
    E value;
    E[] memory values = new E[](1);
    assembly {
      value := 2
    }
    values[0] = value;
    return uint8(values[0]);
  }

  function loadMemoryEnumInvalid() public pure returns (uint8) {
    E[] memory values = new E[](1);
    assembly {
      mstore(add(values, 0x20), 3)
    }
    return uint8(values[0]);
  }

  function storeMemoryEnumInvalid() public pure returns (uint8) {
    E value;
    E[] memory values = new E[](1);
    assembly {
      value := 3
    }
    values[0] = value;
    return uint8(values[0]);
  }

  function discardMemoryEnumInvalid() public pure returns (uint8) {
    E[] memory values = new E[](1);
    assembly {
      mstore(add(values, 0x20), 3)
    }
    values[0];
    return 1;
  }

  function loadMemoryAddress() public pure returns (address) {
    address[] memory values = new address[](1);
    assembly {
      mstore(add(values, 0x20), or(shl(200, 1), 0x1234123412341234123412341234123412341234))
    }
    return values[0];
  }

  function storeMemoryAddress() public pure returns (address) {
    address value;
    address[] memory values = new address[](1);
    assembly {
      value := or(shl(200, 1), 0x1234123412341234123412341234123412341234)
    }
    values[0] = value;
    return values[0];
  }

  function loadMemoryBytes4() public pure returns (bytes4) {
    bytes4[] memory values = new bytes4[](1);
    assembly {
      mstore(add(values, 0x20), 0x12345678ffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
    }
    return values[0];
  }

  function storeMemoryBytes4() public pure returns (bytes4) {
    bytes4 value;
    bytes4[] memory values = new bytes4[](1);
    assembly {
      value := 0x12345678ffffffffffffffffffffffffffffffffffffffffffffffffffffffff
    }
    values[0] = value;
    return values[0];
  }

  function loadMemoryBytes31() public pure returns (bytes31) {
    bytes31[] memory values = new bytes31[](1);
    assembly {
      mstore(add(values, 0x20), 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fff)
    }
    return values[0];
  }

  function storeMemoryBytes31() public pure returns (bytes31) {
    bytes31 value;
    bytes31[] memory values = new bytes31[](1);
    assembly {
      value := 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fff
    }
    values[0] = value;
    return values[0];
  }

  function loadMemoryFnPtr() public view returns (uint) {
    function(uint, uint) external pure returns (uint)[] memory values =
      new function(uint, uint) external pure returns (uint)[](1);
    values[0] = this.add;
    assembly {
      mstore(add(values, 0x20), or(mload(add(values, 0x20)), 1))
    }
    return values[0](20, 22);
  }

  function storeMemoryFnPtr() public view returns (bool) {
    function(uint, uint) external pure returns (uint) value = this.add;
    function(uint, uint) external pure returns (uint)[] memory values =
      new function(uint, uint) external pure returns (uint)[](1);
    assembly {
      value.address := or(value.address, shl(160, sub(0, 1)))
      value.selector := or(value.selector, shl(32, sub(0, 1)))
    }
    values[0] = value;
    uint raw;
    assembly {
      raw := mload(add(values, 0x20))
    }
    return uint64(raw) == 0;
  }
}
