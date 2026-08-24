//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "readAll((uint256,uint256[2],uint256))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "42",
//!                         "1",
//!                         "2",
//!                         "23"
//!                     ],
//!                     "expected": [
//!                         "42",
//!                         "1",
//!                         "2",
//!                         "23"
//!                     ]
//!                 },
//!                 {
//!                     "method": "readC((uint256,uint256[2],uint256))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "100",
//!                         "200",
//!                         "999"
//!                     ],
//!                     "expected": [
//!                         "999"
//!                     ]
//!                 },
//!                 {
//!                     "method": "readC((uint256,uint256[2],uint256))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0",
//!                         "42",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Test {
    struct S {
        uint256 a;
        uint256[2] b;
        uint256 c;
    }

    function readAll(S calldata s)
        external pure
        returns (uint256 a, uint256 b0, uint256 b1, uint256 c)
    {
        a = s.a;
        b0 = s.b[0];
        b1 = s.b[1];
        c = s.c;
    }

    function readC(S calldata s) external pure returns (uint256) {
        return s.c;
    }
}
