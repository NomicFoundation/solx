//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "test()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "5",
//!                         "10"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

struct Data {
    uint value;
}

contract A {
    function get() public pure returns (Data memory) {
        return Data(5);
    }
}

contract Test {
    uint x = 10;
    uint y = 10;

    modifier updateStorage() {
        A a = new A();
        x = a.get().value;
        _;
        y = a.get().value;
    }

    function test() public updateStorage returns (uint, uint) {
        return (x, y);
    }
}
