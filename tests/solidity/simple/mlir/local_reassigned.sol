//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "3",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "14",
//!                         "11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "g(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "13"
//!                     ]
//!                 },
//!                 {
//!                     "method": "h(bool,uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "3",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "h(bool,uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "3",
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i(bool,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "6"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i(bool,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "j(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "k(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "k(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "101"
//!                     ]
//!                 },
//!                 {
//!                     "method": "k(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "200"
//!                     ]
//!                 },
//!                 {
//!                     "method": "k(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "l(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "10",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "l(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "20"
//!                     ],
//!                     "expected": [
//!                         "21",
//!                         "9"
//!                     ]
//!                 },
//!                 {
//!                     "method": "q(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "o(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "o(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "10"
//!                     ],
//!                     "expected": [
//!                         "30"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function f(uint256 a, uint256 b) external pure returns (uint256, uint256) {
        uint256 x = a;
        uint256 y = x + b;
        x = y * 2;
        y = x - a;
        return (x, y);
    }

    function g(uint256 a) external pure returns (uint256) {
        uint256[] memory x = new uint256[](1);
        x[0] = a;
        uint256[] memory y = x;
        x = new uint256[](2);
        x[1] = a + 1;
        return y[0] + x[1] + x.length;
    }

    function h(bool c, uint256 a, uint256 b) external pure returns (uint256) {
        uint256 x = a;
        if (c) {
            x = b;
        } else {
            x = a + b;
        }
        return x;
    }

    modifier m() {
        _;
    }

    function i(bool c, uint256 a) external pure m returns (uint256 x) {
        x = a;
        if (c) {
            return x + 1;
        }
        x = x * 2;
    }

    function j(uint256 a) external pure m returns (uint256 x) {
        x = a;
        x += 3;
    }

    function k(uint256 a) external returns (uint256 x) {
        S s = new S();
        x = 1;
        try s.f(a) returns (uint256 y) {
            x = y;
        } catch Error(string memory) {
            x = 200;
        } catch Panic(uint256 code) {
            x = 100 + code;
        } catch {
            x = 7;
        }
    }

    function l(uint256 n) external pure returns (uint256 x, uint256 y) {
        for (uint256 i = 0; i < n; i++) {
            if (i == 7) break;
            x += i;
            if (i % 2 == 0) continue;
            y += i;
        }
    }

    function q(uint256 n) external pure returns (uint256 x) {
        uint256 i = n;
        do {
            x += i;
            i--;
        } while (i > 0);
    }

    function o(uint256 n) external pure m returns (uint256 x) {
        for (uint256 i = 0; i < n; i++) {
            x += i;
            if (x > 10) return x * 2;
        }
    }
}

contract S {
    function f(uint256 a) external pure returns (uint256) {
        require(a != 0);
        require(a != 2, "two");
        assert(a != 1);
        return a * 2;
    }
}
