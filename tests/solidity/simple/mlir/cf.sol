//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "for_brk(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "20",
//!                         "10"
//!                     ],
//!                     "expected": [
//!                         "1024"
//!                     ]
//!                 },
//!                 {
//!                     "method": "while_cont(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "10"
//!                     ],
//!                     "expected": [
//!                         "22"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern(bool,uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "10",
//!                         "20"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern(bool,uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "10",
//!                         "20"
//!                     ],
//!                     "expected": [
//!                         "20"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_bytes(bool,bytes4,bytes4)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd00000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_bytes(bool,bytes4,bytes4)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000",
//!                         "0xaabbccdd00000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0xaabbccdd00000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_cast(bool,uint8,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "42",
//!                         "1000"
//!                     ],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_cast(bool,uint8,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "42",
//!                         "1000"
//!                     ],
//!                     "expected": [
//!                         "1000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_short_circuit(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_short_circuit(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_tuple(bool,uint256,uint256,uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "40"
//!                     ],
//!                     "expected": [
//!                         "10",
//!                         "20"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_tuple(bool,uint256,uint256,uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "40"
//!                     ],
//!                     "expected": [
//!                         "30",
//!                         "40"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_const(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "tern_const(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint public counter;

  function for_brk(uint a, uint b) public returns (uint) {
    uint r = 1;
    for (uint i = 0; i < a; ++i) {
      if (i == b)
        break;
      r += r;
    }
    return r;
  }

  function while_cont(uint a) public returns (uint) {
    uint r = 1;
    do {
      r = 2;
    } while (false);

    uint i = 0;
    while (i < a) {
      if (i < 99) {
        r += 2;
        i += 1;
        continue;
      }
      r = 3;
    }
    return r;
  }

  function tern(bool c, uint a, uint b) public pure returns (uint) {
    return c ? a : b;
  }

  function tern_bytes(bool c, bytes4 a, bytes4 b) public pure returns (bytes4) {
    return c ? a : b;
  }

  function tern_cast(bool c, uint8 a, uint256 b) public pure returns (uint256) {
    return c ? a : b;
  }

  function tern_short_circuit(bool c) public returns (uint, uint) {
    counter = 0;
    uint result = c ? ++counter : ++counter;
    return (result, counter);
  }

  function tern_tuple(bool c, uint a, uint b, uint x, uint y) public pure returns (uint, uint) {
    return c ? (a, b) : (x, y);
  }

  function tern_const(bool c) public pure returns (uint) {
    return c ? 1 : 2;
  }
}
