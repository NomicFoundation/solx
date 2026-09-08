//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "store",
//!             "calldata": [],
//!             "expected": []
//!         },
//!         {
//!             "method": "copy",
//!             "calldata": [],
//!             "expected": [
//!                 "7"
//!             ]
//!         },
//!         {
//!             "method": "clear",
//!             "calldata": [],
//!             "expected": []
//!         },
//!         {
//!             "method": "copy",
//!             "calldata": [],
//!             "expected": [
//!                 "0"
//!             ]
//!         }
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// The cycle is closed by a function type alone, which solc does not follow, so solx identifies the
// struct that solc's `recursive` annotation leaves unmarked. Each call runs in its own transaction,
// so the copy out of storage and the clearing of it read and write storage rather than folding
// within one body.

pragma solidity >=0.8.0;

contract Test {
    struct Closure {
        function(Closure memory) internal pure returns (uint256) call;
        uint256 tag;
    }

    Closure stored;

    function store() public {
        stored.tag = 7;
    }

    function copy() public view returns (uint256) {
        Closure memory snapshot = stored;
        return snapshot.tag;
    }

    function clear() public {
        delete stored;
    }
}
