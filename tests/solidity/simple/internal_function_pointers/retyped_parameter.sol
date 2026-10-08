//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "f",
//!             "calldata": [ "3", "4" ]
//!         }
//!     ],
//!     "expected": [ "7" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

struct S {
    uint256 a;
    uint256 b;
}

// A function taking a memory pointer, called through a pointer retyped to take the struct it points
// to.
contract Test {
    function f(uint256 a, uint256 b) external pure returns (uint256) {
        return h(g)(S(a, b));
    }

    function g(uint256 p) private pure returns (uint256 x) {
        assembly {
            x := add(mload(p), mload(add(p, 0x20)))
        }
    }

    function h(function(uint256) pure returns (uint256) x)
        private
        pure
        returns (function(S memory) pure returns (uint256) y)
    {
        assembly {
            y := x
        }
    }
}
