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

// A function returning a memory pointer, called through a pointer retyped to return the struct it
// points to.
contract Test {
    function f(uint256, uint256) external pure returns (uint256, uint256) {
        S memory s = h(g)(4);
        return (s.a, s.b);
    }

    function g(uint256 x) private pure returns (uint256 p) {
        assembly {
            p := mload(0x40)
            calldatacopy(p, x, 0x40)
            mstore(0x40, add(p, 0x40))
        }
    }

    function h(function(uint256) pure returns (uint256) x)
        private
        pure
        returns (function(uint256) pure returns (S memory) y)
    {
        assembly {
            y := x
        }
    }
}
