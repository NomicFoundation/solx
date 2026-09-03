//! { "modes": [ "E" ], "cases": [ {
//!     "name": "user_function",
//!     "inputs": [ { "method": "user_function", "calldata": [] } ],
//!     "expected": [ "321" ]
//! }, {
//!     "name": "builtin",
//!     "inputs": [ { "method": "builtin", "calldata": [] } ],
//!     "expected": [ "21" ]
//! }, {
//!     "name": "nested",
//!     "inputs": [ { "method": "nested", "calldata": [] } ],
//!     "expected": [ "4321" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function user_function() external pure returns (uint256) {
        assembly {
            function step(tag) -> ret {
                mstore(0, add(mul(mload(0), 10), tag))
                ret := 0
            }
            function sink(a, b, c) {
                pop(add(add(a, b), c))
            }
            mstore(0, 0)
            sink(step(1), step(2), step(3))
            mstore(32, mload(0))
            return(32, 32)
        }
    }

    function builtin() external pure returns (uint256) {
        assembly {
            function step(tag) -> ret {
                mstore(0, add(mul(mload(0), 10), tag))
                ret := 0
            }
            mstore(0, 0)
            mstore(add(step(1), 64), step(2))
            mstore(96, mload(0))
            return(96, 32)
        }
    }

    function nested() external pure returns (uint256) {
        assembly {
            function step(tag) -> ret {
                mstore(0, add(mul(mload(0), 10), tag))
                ret := 0
            }
            function pair(a, b) -> ret {
                ret := add(a, b)
            }
            mstore(0, 0)
            pop(pair(pair(step(1), step(2)), pair(step(3), step(4))))
            mstore(32, mload(0))
            return(32, 32)
        }
    }
}
