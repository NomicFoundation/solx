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
//!                             "0x071cef01c6c16428086bed395f60f8f569a0b85b25c29f7e32226b6d2ef21e70"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x51f81bcdfc324a0dff2b5bec9d92e21cbebc4d5e29d3a3d30de3e03fbeab8d7f",
//!                             "0xf9b886ea4fc1f121c4fbec868fd183bdc1c5ed2d295c23c4275767a44470ac49",
//!                             "0xe88616e3ab9b2cb483ad6de006f73fbb98e515bcfa461ee349054eec12d7ab8e"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "h()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                             "0x0000000000000000000000000000000000000000000000000000000000001234",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                             "0x0102030400000000000000000000000000000000000000000000000000000000",
//!                             "0x000000000000000000000000000000000000000000000000000000000000002d"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "k()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000008",
//!                             "0x000000000000000000000000000000000000000000000000000000000000000b"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "l()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000007"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "m()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000021",
//!                             "0x9aec33d182d07e885af7e75821c21b3faf468a0f0b3432d7009454569fbfb3a9"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "n()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000003",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000009",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000001"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "q()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000000",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000007",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000005",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000002"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "r()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "p()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x117f467b7466ecf3ca7fcef8ab84ef083538a4f63454d1748a73dd9f73730af4"
//!                         ]
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

interface I {
    function b() external returns (uint256);
}

contract Test {
    struct S {
        uint256 a;
        uint256 b;
    }

    function d(uint256 x) external pure {
        assembly {
            for { let i := 0 } lt(i, add(x, 32)) { i := add(i, 32) } { mstore(i, not(i)) }
            return(0, x)
        }
    }

    function a() external pure returns (uint256, address, bool, bytes4) {
        return (7, address(0x1234), true, 0x01020304);
    }

    function b() external pure returns (uint256, uint256) {
        return (8, 9);
    }

    function c() external pure returns (uint256[2] memory) {
        return [uint256(5), 6];
    }

    function e(bool x) internal pure returns (S memory) {
        if (x) {
            return S(1, 2);
        }
    }

    function j(bool x) internal pure returns (S memory, uint256, bytes memory) {
        if (x) {
            return (S(3, 4), 5, hex"0102");
        }
    }

    modifier skip() {
        if (false) {
            _;
        }
    }

    function o() internal pure skip returns (S memory) {
        return S(7, 8);
    }

    function f() public pure returns (bytes32 x) {
        for (uint256 i = 0; i < 3; i++) {
            bytes memory y = new bytes(i);
            x = keccak256(abi.encode(x, i, y));
        }
    }

    function g() public pure returns (bytes32 x, bytes32 y, bytes32 z) {
        bytes memory w = new bytes(3);
        x = keccak256(abi.encodePacked(w, uint8(1)));
        y = keccak256(abi.encodeWithSelector(0x12345678, w));
        bytes memory t = abi.encode(w);
        z = keccak256(t);
        t[0] = 0x01;
        z = z ^ keccak256(t);
    }

    function h() public returns (uint256 x, address y, bool z, bytes4 w, uint256 v) {
        (x, y, z, w) = this.a();
        bytes memory u = new bytes(40);
        u[39] = 0x05;
        v = uint8(u[39]) + u.length;
    }

    function k() public returns (uint256 x, uint256 y) {
        x = I(address(this)).b();
        uint256[2] memory u = this.c();
        bytes memory v = new bytes(64);
        v[0] = 0x01;
        y = u[0] + u[1];
    }

    function l() public returns (bool ok, uint256 x) {
        (ok, ) = address(this).call(abi.encodeWithSignature("d(uint256)", uint256(33)));
        bytes memory u = new bytes(3);
        u[2] = 0x07;
        x = uint8(u[2]);
    }

    function m() public returns (bool ok, uint256 x, bytes32 y) {
        (bool u, bytes memory t) = address(this).call(abi.encodeWithSignature("d(uint256)", uint256(33)));
        ok = u;
        x = t.length;
        y = keccak256(t);
    }

    function n() public pure returns (uint256 x, uint256 y, uint256 z, uint256 w) {
        S memory s = e(false);
        S memory t = e(true);
        x = s.a + s.b;
        y = t.a + t.b;
        s.a = 9;
        z = s.a;
        w = t.a;
    }

    function q() public pure returns (uint256 x, uint256 y, uint256 z, uint256 w) {
        (S memory s, uint256 u, bytes memory v) = j(false);
        x = s.a + s.b + u + v.length;
        (s, u, v) = j(true);
        y = s.a + s.b;
        z = u;
        w = v.length;
    }

    function r() public pure returns (uint256 x) {
        S memory s = o();
        x = s.a + s.b;
    }

    function p() public returns (bytes32 x) {
        for (uint256 i = 0; i < 3; i++) {
            x = keccak256(abi.encode(x, I(address(this)).b()));
        }
    }
}
