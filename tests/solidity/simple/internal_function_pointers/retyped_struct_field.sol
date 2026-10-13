//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "f",
//!             "calldata": [ "7", "9" ]
//!         }
//!     ],
//!     "expected": [ "7", "9" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

struct S {
    uint256 a;
    uint256 b;
}

struct T {
    function(uint256) pure returns (uint256) f;
}

struct U {
    function(uint256) pure returns (S memory) f;
}

// A pointer held in a struct field, retyped with the whole struct.
contract Test {
    function f(uint256, uint256) external pure returns (uint256, uint256) {
        S memory s = h(T(g)).f(4);
        return (s.a, s.b);
    }

    function g(uint256 x) private pure returns (uint256 p) {
        assembly {
            p := mload(0x40)
            calldatacopy(p, x, 0x40)
            mstore(0x40, add(p, 0x40))
        }
    }

    function h(T memory x) private pure returns (U memory y) {
        assembly {
            y := x
        }
    }
}
