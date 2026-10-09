//! { "cases": [ {
//!     "name": "array_through_words",
//!     "inputs": [
//!         {
//!             "method": "f",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [ "5" ]
//! }, {
//!     "name": "words_through_array",
//!     "inputs": [
//!         {
//!             "method": "g",
//!             "calldata": [ "0x20", "3", "7", "8", "9" ]
//!         }
//!     ],
//!     "expected": [ "3" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// A calldata array is two words, its offset and length, so a pointer retyped between it and a pair of
// words reaches either function.
contract Test {
    function f() public pure returns (uint256) {
        function(uint256[] calldata) pure returns (uint256[] calldata) x = h;
        function(uint256, uint256) pure returns (uint256, uint256) y;
        assembly {
            y := x
        }
        (, uint256 b) = y(4, 5);
        return b;
    }

    function g(uint256[] calldata a) public pure returns (uint256) {
        function(uint256, uint256) pure returns (uint256, uint256) x = k;
        function(uint256[] calldata) pure returns (uint256[] calldata) y;
        assembly {
            y := x
        }
        return y(a).length;
    }

    function h(uint256[] calldata a) private pure returns (uint256[] calldata) {
        return a;
    }

    function k(uint256 a, uint256 b) private pure returns (uint256, uint256) {
        return (a, b);
    }
}
