//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "u24CrossSlotPop()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "900"
//!                     ]
//!                 },
//!                 {
//!                     "method": "u24WithinSlotPop()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "800"
//!                     ]
//!                 },
//!                 {
//!                     "method": "u72CrossSlotPop()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "3"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Test {
	uint24[] u24a;
	uint24[] u24b;
	uint72[] u72;

	// Push 11 uint24 values (indices 0-10).  Index 10 lands in slot 1
	// (10 / 10 = 1).  Pop removes index 10.  Element 9 (last of slot 0,
	// bits 216-239) must be unchanged at 900.
	function u24CrossSlotPop() public returns (uint24) {
		for (uint i = 0; i < 11; i++)
			u24a.push(uint24(i * 100));
		u24a.pop();
		return u24a[9];
	}

	// Push 10 uint24 values (fills slot 0 exactly).  Pop removes index 9
	// (9 / 10 = 0, within slot 0).  Element 8 must be unchanged at 800.
	function u24WithinSlotPop() public returns (uint24) {
		for (uint i = 0; i < 10; i++)
			u24b.push(uint24(i * 100));
		u24b.pop();
		return u24b[8];
	}

	// Push 4 uint72 values (indices 0-3).  Index 3 lands in slot 1
	// (3 / 3 = 1).  Pop removes index 3.  Element 2 (last of slot 0) must
	// be unchanged at 3.
	function u72CrossSlotPop() public returns (uint72) {
		for (uint i = 0; i < 4; i++)
			u72.push(uint72(i + 1));
		u72.pop();
		return u72[2];
	}
}
