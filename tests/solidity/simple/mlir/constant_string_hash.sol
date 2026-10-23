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
//!                             "0x4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000080"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45",
//!                             "0x4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45",
//!                             "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
//!                             "0x3540a565d3691b7f4c1f2fde1cc64f39c76c7296b38b01bc5acea259986fe043"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "h()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0xba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
//!                             "0x8eb208f7e05d987a9b044a8e98c6b087f15a0bfc000000000000000000000000",
//!                             "0x541ecd394b8706ed954ccc08a66b5d6f10a6379bbcacc92ed87f0884c5389853",
//!                             "0x9c1185a5c5e9fc54612808977ee8f548b2258d31000000000000000000000000"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "k()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "l()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45",
//!                             "0xeee2aaf48e96e3635ad10cd1e390feec918895c21edf42178b66eff35bcc6a00",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000060",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                             "0x6162630000000000000000000000000000000000000000000000000000000000"
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
    string constant C = "abc";
    bytes32 constant S = keccak256(abi.encode(uint256(keccak256(abi.encodePacked("x.y"))) - 1)) & ~bytes32(uint256(0xff));

    function f() public pure returns (bytes32 h, uint256 p) {
        h = keccak256("abc");
        assembly {
            p := mload(0x40)
        }
    }

    function g() public pure returns (bytes32, bytes32, bytes32, bytes32) {
        return (keccak256(bytes("abc")), keccak256(bytes(C)), keccak256(""), keccak256("abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn"));
    }

    function h() public pure returns (bytes32, bytes20, bytes32, bytes20) {
        return (sha256("abc"), ripemd160("abc"), sha256("abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn"), ripemd160(""));
    }

    function k() public pure returns (bytes32) {
        return keccak256(bytes(string.concat("a", "bc")));
    }

    function l() public pure returns (bytes32, bytes32, bytes memory) {
        return (keccak256(abi.encodePacked("a", "bc")), S, abi.encodePacked("a", "bc"));
    }
}
