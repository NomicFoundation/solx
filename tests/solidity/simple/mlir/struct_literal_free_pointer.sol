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
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000100",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                             "0x00000000000000000000000000000000000000000000000000000000000000c0",
//!                             "0x6e0c627900b24bd432fe7b1f713f1b0744091a646a9fe4a65a18dfed21f2949c"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000100",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                             "0x6e0c627900b24bd432fe7b1f713f1b0744091a646a9fe4a65a18dfed21f2949c"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "h()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000140",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                             "0x392791df626408017a264f53fde61065d5a93a32b60171df9d8a46afdf82992d"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "k()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000140",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                             "0x7ee1fe5c779f64c56cb79d51c57433762d8096174ff21f3418a5bec4efa475af"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "l()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000160",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080",
//!                             "0x392791df626408017a264f53fde61065d5a93a32b60171df9d8a46afdf82992d"
//!                         ]
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    struct S {
        uint256 a;
        uint256 b;
    }

    struct N {
        S s;
        uint256 c;
    }

    struct M {
        uint256[2] a;
        uint256 b;
    }

    struct O {
        N n;
        uint256 d;
    }

    struct T {
        S s;
        string t;
    }

    struct A {
        S[2] a;
    }

    function f() public pure returns (uint256 p, uint256 q, uint256 r, bytes32 z) {
        N memory x = N(S(1, 2), 3);
        S memory y = x.s;
        assembly {
            p := mload(0x40)
            q := x
            r := y
        }
        z = keccak256(abi.encode(x));
    }

    function g() public pure returns (uint256 p, uint256 q, bytes32 z) {
        M memory x = M([uint256(1), 2], 3);
        assembly {
            p := mload(0x40)
            q := x
        }
        z = keccak256(abi.encode(x));
    }

    function h() public pure returns (uint256 p, uint256 q, bytes32 z) {
        O memory x = O(N(S(1, 2), 3), 4);
        assembly {
            p := mload(0x40)
            q := x
        }
        z = keccak256(abi.encode(x));
    }

    function k() public pure returns (uint256 p, uint256 q, bytes32 z) {
        T memory x = T(S(1, 2), "ab");
        assembly {
            p := mload(0x40)
            q := x
        }
        z = keccak256(abi.encode(x));
    }

    function l() public pure returns (uint256 p, uint256 q, bytes32 z) {
        A memory x = A([S(1, 2), S(3, 4)]);
        assembly {
            p := mload(0x40)
            q := x
        }
        z = keccak256(abi.encode(x));
    }
}
