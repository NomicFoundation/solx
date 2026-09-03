//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "init()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "memoryArray(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x100000000000000000000000000000000000000000000000000000000000001"
//!                     ],
//!                     "expected": [
//!                         "22"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memoryBytes(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x100000000000000000000000000000000000000000000000000000000000001"
//!                     ],
//!                     "expected": [
//!                         "0x2200000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storageStatic(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x100000000000000000000000000000000000000000000000000000000000001"
//!                     ],
//!                     "expected": [
//!                         "22"
//!                     ]
//!                 },
//!                 {
//!                     "method": "storageDynamic(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x100000000000000000000000000000000000000000000000000000000000001"
//!                     ],
//!                     "expected": [
//!                         "22"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256[4] s;
    uint256[] d;

    function init() public {
        s[0] = 11;
        s[1] = 22;
        s[2] = 33;
        s[3] = 44;
        d.push(11);
        d.push(22);
        d.push(33);
        d.push(44);
    }

    function memoryArray(uint256 dirty) public pure returns (uint256) {
        uint8 i;
        assembly { i := dirty }
        uint256[4] memory a;
        a[0] = 11;
        a[1] = 22;
        a[2] = 33;
        a[3] = 44;
        return a[i];
    }

    function memoryBytes(uint256 dirty) public pure returns (bytes1) {
        uint8 i;
        assembly { i := dirty }
        bytes memory b = new bytes(4);
        b[0] = 0x11;
        b[1] = 0x22;
        b[2] = 0x33;
        b[3] = 0x44;
        return b[i];
    }

    function storageStatic(uint256 dirty) public view returns (uint256) {
        uint8 i;
        assembly { i := dirty }
        return s[i];
    }

    function storageDynamic(uint256 dirty) public view returns (uint256) {
        uint8 i;
        assembly { i := dirty }
        return d[i];
    }
}
