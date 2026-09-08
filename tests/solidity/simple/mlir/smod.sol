//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "mod(int256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "7",
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "mod(int256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "-7",
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "-2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "mod(int256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "7",
//!                         "-5"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "mod(int256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "-7",
//!                         "-5"
//!                     ],
//!                     "expected": [
//!                         "-2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "mod(int256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000001200000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "mod(int256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "-57896044618658097711785492504343953926634992332820282019728792003956564819968",
//!                         "-1"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "modU(int256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "-7",
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "-2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "modU(int256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000001200000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "modU(int256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "-57896044618658097711785492504343953926634992332820282019728792003956564819968",
//!                         "-1"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "mod8U(int8,int8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "-7",
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "-2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "mod8U(int8,int8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "-128",
//!                         "-1"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "modImm()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "modImmMinusOne()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "modImmUncheckedMinusOne()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
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
  function mod(int a, int b) public pure returns (int) {
    return a % b;
  }
  function modU(int a, int b) public pure returns (int) {
    unchecked { return a % b; }
  }
  function mod8U(int8 a, int8 b) public pure returns (int8) {
    unchecked { return a % b; }
  }

  function modImm() public pure returns (int) {
    return int(-7) % 5;
  }
  function modImmMinusOne() public pure returns (int) {
    int x = type(int).min;
    return x % -1;
  }
  function modImmUncheckedMinusOne() public pure returns (int) {
    int x = type(int).min;
    unchecked { return x % -1; }
  }
}
