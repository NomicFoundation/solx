//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x20",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000006",
//!                         "0x6162636162630000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "h()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000006",
//!                         "0x6162636162630000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function f() public returns (string memory) {
        string memory b = "";
        return string.concat(
            string.concat(b),
            string.concat(b, b),
            string.concat("", b),
            string.concat(b, "")
        );
    }

    function g() public returns (string memory) {
        return string.concat("", "abc", hex"", "abc", unicode"");
    }

    function h() public returns (string memory) {
        string memory b = "";
        return string.concat(b, "abc", b, "abc", b);
    }
}
