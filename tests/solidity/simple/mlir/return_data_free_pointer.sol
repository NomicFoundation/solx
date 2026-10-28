//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000060"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000060"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "h()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000060"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "k()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000060"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "l()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x00000000000000000000000000000000000000000000000000000000000000e0",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000040",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                             "0x6100000000000000000000000000000000000000000000000000000000000000"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "m()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080"
//!                         ]
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    fallback(bytes calldata x) external returns (bytes memory) {
        return x;
    }

    function a() external pure {
        revert();
    }

    function b(uint256 x) external pure {
        assembly {
            mstore(0, shl(224, 0x08c379a0))
            mstore(4, x)
            mstore(0x24, 1)
            mstore(0x44, shl(248, 0x61))
            revert(0, 0xa4)
        }
    }

    function f() public returns (uint256 p, uint256 q) {
        (, bytes memory x) = address(this).call("");
        assembly {
            p := mload(0x40)
            q := x
        }
    }

    function g() public returns (uint256 p, uint256 q) {
        (, bytes memory x) = address(this).staticcall("");
        assembly {
            p := mload(0x40)
            q := x
        }
    }

    function h() public returns (uint256 p, uint256 q) {
        (, bytes memory x) = address(this).delegatecall("");
        assembly {
            p := mload(0x40)
            q := x
        }
    }

    function k() public returns (uint256 p, uint256 q) {
        try this.a() {} catch (bytes memory x) {
            assembly {
                p := mload(0x40)
                q := x
            }
        }
    }

    function l() public returns (uint256 p, string memory y) {
        try this.b(0x20) {} catch Error(string memory x) {
            assembly {
                p := mload(0x40)
            }
            y = x;
        }
    }

    function m() public returns (uint256 p) {
        try this.b(1 << 64) {} catch Error(string memory) {} catch {
            assembly {
                p := mload(0x40)
            }
        }
    }
}
