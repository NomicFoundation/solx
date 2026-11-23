//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "up",
//!             "inputs": [
//!                 {
//!                     "method": "g",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "11"
//!                     ]
//!                 }
//!             ]
//!         },
//!         {
//!             "name": "down",
//!             "inputs": [
//!                 {
//!                     "method": "g",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "9"
//!                     ]
//!                 }
//!             ]
//!         },
//!         {
//!             "name": "context",
//!             "inputs": [
//!                 {
//!                     "method": "h",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ],
//!     "libraries": {
//!         "library_delegate_pointer.sol": {
//!             "L": "L"
//!         }
//!     }
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

library L {
    function up(uint256[] storage r) external {
        r[0] += 1;
    }

    function down(uint256[] storage r) external {
        r[0] -= 1;
    }

    function self(uint256) external view returns (address) {
        return address(this);
    }

    function zero(uint256) external pure returns (address) {
        return address(0);
    }
}

contract Test {
    uint256[] r;

    constructor() {
        r.push(10);
    }

    function g(bool b) external returns (uint256) {
        (b ? L.up : L.down)(r);
        return r[0];
    }

    function h(bool b) external view returns (bool) {
        return (b ? L.self : L.zero)(0) == address(this);
    }
}
