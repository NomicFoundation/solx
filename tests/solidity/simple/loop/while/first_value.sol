//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "whileLoop",
//!             "calldata": [
//!                 "3"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1", "3"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// TODO: Fails at -Oz if mergebb merges away a block whose value is used outside it.
contract Test {
    uint256 counter;
    uint256 other;

    function next() internal returns (uint256 r) {
        counter += 1;
        r = counter;
    }

    function whileLoop(uint256 x) external returns (uint256 first, uint256 last) {
        if (x > 100) other = 1;
        first = next();
        last = first;
        while (last < x) {
            last = next();
        }
    }
}
