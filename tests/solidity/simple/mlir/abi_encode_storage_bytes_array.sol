//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "dynamic20()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "13107"
//!                     ]
//!                 },
//!                 {
//!                     "method": "static20()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "17476"
//!                     ]
//!                 },
//!                 {
//!                     "method": "static31()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "21845"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    bytes20[] list20;
    bytes20[2] pair20;
    bytes31[2] pair31;

    constructor() {
        list20.push(bytes20(uint160(0x3333)));
        pair20[1] = bytes20(uint160(0x4444));
        pair31[1] = bytes31(uint248(0x5555));
    }

    function dynamic20() external view returns (uint256) {
        bytes20[] memory m = abi.decode(abi.encode(list20), (bytes20[]));
        return uint160(m[0]);
    }

    function static20() external view returns (uint256) {
        bytes20[2] memory m = abi.decode(abi.encode(pair20), (bytes20[2]));
        return uint160(m[1]);
    }

    function static31() external view returns (uint256) {
        bytes31[2] memory m = abi.decode(abi.encode(pair31), (bytes31[2]));
        return uint248(m[1]);
    }
}
