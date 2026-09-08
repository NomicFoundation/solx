//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "readAll((uint256,uint256[3],uint256[2],uint256))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "100",
//!                         "200",
//!                         "999"
//!                     ],
//!                     "expected": [
//!                         "1",
//!                         "30",
//!                         "200",
//!                         "999"
//!                     ]
//!                 },
//!                 {
//!                     "method": "readD((uint256,uint256[3],uint256[2],uint256))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "100",
//!                         "200",
//!                         "999"
//!                     ],
//!                     "expected": [
//!                         "999"
//!                     ]
//!                 },
//!                 {
//!                     "method": "readD((uint256,uint256[3],uint256[2],uint256))",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0",
//!                         "0",
//!                         "42",
//!                         "0",
//!                         "0",
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
        uint256[3] b;
        uint256[2] c;
        uint256 d;
    }

    function readAll(S calldata s)
        external pure
        returns (uint256 a, uint256 b2, uint256 c1, uint256 d)
    {
        a = s.a;
        b2 = s.b[2];
        c1 = s.c[1];
        d = s.d;
    }

    function readD(S calldata s) external pure returns (uint256) {
        return s.d;
    }
}
