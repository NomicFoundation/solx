//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "2",
//!                         "7",
//!                         "12"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ],
//!     "libraries": {
//!         "library_storage_ref_abi.sol": {
//!             "L": "L"
//!         }
//!     }
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

library L {
    struct S {
        uint x;
        uint y;
    }

    function f(uint[] storage r, S storage s)
        external
        view
        returns (uint, uint, uint, uint)
    {
        return (r[0], r[1], s.x, s.y);
    }
}

contract Test {
    L.S s;
    uint[] r;

    constructor() {
        r.push(1);
        r.push(2);
        s.x = 7;
        s.y = 12;
    }

    function g() external view returns (uint, uint, uint, uint) {
        return L.f(r, s);
    }
}
