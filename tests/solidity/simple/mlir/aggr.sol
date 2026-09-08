//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "m(address,address,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ar(uint256[2])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "2",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ar2(uint256[2][2])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "3",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "arr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dar2(uint256[][])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "2",
//!                         "0x40",
//!                         "0xa0",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "2",
//!                         "5",
//!                         "6"
//!                     ],
//!                     "expected": [
//!                         "5",
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dar3(uint256[2][])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "2",
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "3",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "darr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "32",
//!                         "2",
//!                         "1",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "darr2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "2",
//!                         "0x40",
//!                         "0xa0",
//!                         "2",
//!                         "0",
//!                         "1",
//!                         "2",
//!                         "1",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "darr3()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "1",
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dcarr(uint256[][])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "2",
//!                         "0x40",
//!                         "0xa0",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "2",
//!                         "5",
//!                         "6"
//!                     ],
//!                     "expected": [
//!                         "0x20",
//!                         "2",
//!                         "0x40",
//!                         "0xa0",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "2",
//!                         "5",
//!                         "6"
//!                     ]
//!                 },
//!                 {
//!                     "method": "car(uint256[2])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dcar(uint256[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "32",
//!                         "2",
//!                         "1",
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "len()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "lit()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "lit2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "10",
//!                         "11",
//!                         "20",
//!                         "21",
//!                         "30",
//!                         "31"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  mapping(address => uint) private m0;
  mapping(address => mapping(address => uint)) private m1;

  function m(address a, address b, uint c) public returns (uint) {
    m0[a] = c;
    m1[a][b] = c + 2;
    return m0[a] + m1[a][b];
  }

  function ar(uint[2] memory a) public returns (uint, uint) {
    a[0] = a[1];
    return (a[0], a[1]);
  }
  function ar2(uint[2][2] memory a) public returns (uint, uint) {
    a[0][1] = a[1][0];
    return (a[0][1], a[1][0]);
  }
  function arr() public returns (uint[2] memory) {
    uint[2] memory a;
    a[0] = 1;
    a[1] = a[0];
    return a;
  }

  function dar(uint[] memory a) public returns (uint, uint) {
    a[0] = a[1];
    return (a[0], a[1]);
  }
  function dar2(uint[][] memory a) public returns (uint, uint) {
    a[0][1] = a[1][0];
    return (a[0][1], a[1][0]);
  }
  function dar3(uint[2][] memory a) public returns (uint, uint) {
    a[0][1] = a[1][0];
    return (a[0][1], a[1][0]);
  }
  function darr() public returns (uint[] memory) {
    uint[] memory a;
    a = new uint[](2);
    a[0] = 1;
    a[1] = a[0];
    return a;
  }
  function darr2() public returns (uint[][] memory) {
    uint[][] memory a;
    a = new uint[][](2);
    a[0] = new uint[](2);
    a[1] = new uint[](2);
    a[0][1] = 1;
    a[1][0] = a[0][1];
    return a;
  }
  function darr3() public returns (uint[2][] memory) {
    uint[2][] memory a;
    a = new uint[2][](1);
    return a;
  }

  function car(uint[2] calldata a) public returns (uint) {
    return a.length + a[0];
  }

  function dcar(uint[] calldata a) public returns (uint) {
    return a.length + a[0];
  }

  function dcarr(uint[][] calldata a) public returns (uint[][] calldata) {
    return a;
  }

  function len() public returns (uint, uint) {
    uint[] memory a;
    a = new uint[](2);
    uint[3] memory b;
    return (a.length, b.length);
  }

  function lit() public returns (uint[2] memory) {
    uint b = 2;
    uint[2] memory a = [1, b];
    return a;
  }
  function lit2() public returns (uint[2][3] memory) {
    uint[2][3] memory a = [[uint(10), 11], [uint(20), 21], [uint(30), 31]];
    return a;
  }
}
