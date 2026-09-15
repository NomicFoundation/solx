//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "i()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i2(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "s()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "3",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m(address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m2(address,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "6"
//!                     ]
//!                 },
//!                 {
//!                     "method": "mc(address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "8"
//!                     ]
//!                 },
//!                 {
//!                     "method": "cm(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ci()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint public i;
  uint[2][2] public i2;
  string public str;
  struct S {
    uint i;
    uint8 i8;
  }
  S public s;
  mapping(address => uint) public m;
  mapping(address => mapping(uint => uint)) public m2;
  mapping(Test => uint) public mc;
  mapping(uint => Test) public cm;

  uint public constant ci = 7;

  constructor () {
    i = 1;
    i2[0][0] = 2;
    s.i = 3;
    // FIXME:
    // llvm/lib/Target/EVM/EVMStackModel.cpp:107: void llvm::EVMStackModel::processMI(const MachineInstr &): Assertion
    // `Opc != EVM::STACK_LOAD && Opc != EVM::STACK_STORE && "Unexpected stack memory instruction"' failed.
	  // s.i8 = 4;
    m[address(0)] = 5;
    m2[address(0)][0] = 6;
    mc[Test(address(0))] = 8;
    cm[1] = Test(address(2));
  }
}
