//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "m(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "0x120",
//!                         "0x140",
//!                         "0",
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setA(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setIsigned(uint256,int256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "-7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setSmall(uint256,uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "255"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setFlag(uint256,bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setAddr(uint256,address)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0x1234"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setB32(uint256,bytes32)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0x41"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setB(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "99"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "42",
//!                         "-7",
//!                         "255",
//!                         "1",
//!                         "0x1234",
//!                         "0x41",
//!                         "0x120",
//!                         "0x140",
//!                         "99",
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "setStringHello(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "0x000000000000000000000000000000000000000000000000000000000000002a",
//!                         "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9",
//!                         "0x00000000000000000000000000000000000000000000000000000000000000ff",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                         "0x0000000000000000000000000000000000000000000000000000000000001234",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000041",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000120",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000160",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000063",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000005",
//!                         "0x68656c6c6f000000000000000000000000000000000000000000000000000000",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// Verifies that the auto-generated public getter for a mapping-to-struct
// includes value types (uint256, int256, uint8, bool, address, bytes32,
// string, bytes) and excludes arrays (dynamic and fixed).
// Return type: (uint256,int256,uint8,bool,address,bytes32,string,bytes,uint256)
contract Test {
    struct S {
        uint256    a;
        int256     isigned;
        uint8      small;
        bool       flag;
        address    addr;
        bytes32    b32;
        uint256[]  dynArr;
        string     s;
        bytes      bdata;
        uint256[2] fixArr;
        uint256    b;
    }

    mapping(uint256 => S) public m;

    function setA(uint256 key, uint256 val) public { m[key].a = val; }
    function setIsigned(uint256 key, int256 val) public { m[key].isigned = val; }
    function setSmall(uint256 key, uint8 val) public { m[key].small = val; }
    function setFlag(uint256 key, bool val) public { m[key].flag = val; }
    function setAddr(uint256 key, address val) public { m[key].addr = val; }
    function setB32(uint256 key, bytes32 val) public { m[key].b32 = val; }
    function setB(uint256 key, uint256 val) public { m[key].b = val; }
    function setStringHello(uint256 key) public { m[key].s = "hello"; }
}
