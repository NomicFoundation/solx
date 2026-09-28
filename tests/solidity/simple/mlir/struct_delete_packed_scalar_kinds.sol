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
//!                         "0xffffff0000000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffff00000000000000000000000000000000000000000000000000000000",
//!                         "0xffffffffffffffff000000000000000000000000000000000000000000000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// `delete` on a storage struct whose members are every kind of packable
// scalar.
contract Test {
    enum E { A, B, C }
    struct Mix {
        bool b;
        uint8 u8;
        E e;
        address a;
        bytes4 b4;
        uint16 u16;
        function () internal returns (uint256) fp;
        bytes20 b20;
        function () external returns (uint256) efp;
    }
    Mix m;

    function h() internal pure returns (uint256) { return 1; }
    function g() external pure returns (uint256) { return 2; }

    function f() public returns (uint256 r0, uint256 r1, uint256 r2) {
        assembly {
            sstore(m.slot, not(0))
            sstore(add(m.slot, 1), not(0))
            sstore(add(m.slot, 2), not(0))
        }
        m.b = true; m.u8 = 1; m.e = E.B; m.a = address(1); m.b4 = 0x01020304;
        m.u16 = 7; m.fp = h; m.b20 = bytes20(uint160(2)); m.efp = this.g;
        delete m;
        assert(!m.b);
        assert(m.u8 == 0);
        assert(m.e == E.A);
        assert(m.a == address(0));
        assert(m.b4 == 0);
        assert(m.u16 == 0);
        assert(m.b20 == 0);
        assembly {
            r0 := sload(m.slot)
            r1 := sload(add(m.slot, 1))
            r2 := sload(add(m.slot, 2))
        }
    }
}
