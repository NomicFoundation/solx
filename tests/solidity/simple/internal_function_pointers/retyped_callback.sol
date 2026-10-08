//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "f",
//!             "calldata": [ "3", "4" ]
//!         }
//!     ],
//!     "expected": [ "12" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

struct S {
    uint256 a;
    uint256 b;
}

// A callback passed through a retyped pointer, called through the parameter type of the function
// the pointer reaches.
contract Test {
    function f(uint256 a, uint256 b) external pure returns (uint256) {
        return k(g)(S(a, b), h);
    }

    function g(uint256 p, function(uint256) pure returns (uint256) x) private pure returns (uint256) {
        return x(p);
    }

    function h(S memory s) private pure returns (uint256) {
        return s.a * s.b;
    }

    function k(function(uint256, function(uint256) pure returns (uint256)) pure returns (uint256) x)
        private
        pure
        returns (function(S memory, function(S memory) pure returns (uint256)) pure returns (uint256) y)
    {
        assembly {
            y := x
        }
    }
}
