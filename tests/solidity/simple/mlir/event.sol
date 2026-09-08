//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f(address,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "events": [
//!                             {
//!                                 "topics": [
//!                                     "0xbdd4be579984a3856cd1022b131de0a9912cd0f746e727f0d0a56ef44cab8cc2",
//!                                     "0x01"
//!                                 ],
//!                                 "values": [
//!                                     "0x02"
//!                                 ]
//!                             }
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "dirtyAddress()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [],
//!                         "events": [
//!                             {
//!                                 "topics": [
//!                                     "0xa4f051d1f3aa50f530737102b6182d2f10e3f663d0f1ca8db751332a7a12f2c5",
//!                                     "0xffffffffffffffffffffffffffffffffffffffff"
//!                                 ],
//!                                 "values": []
//!                             }
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "dirtyUint()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [],
//!                         "events": [
//!                             {
//!                                 "topics": [
//!                                     "0x2b7205ef8709370e87d908b974eabddd9cad12bb3627578e9b956868b1f450a6",
//!                                     "0xff"
//!                                 ],
//!                                 "values": []
//!                             }
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
  event E(address indexed a, uint b);
  event DirtyAddress(address indexed a);
  event DirtyUint(uint8 indexed a);

  function f(address a, uint b) public {
    emit E(a, b);
  }

  function dirtyAddress() public {
    address a;
    assembly {
      a := not(0)
    }
    emit DirtyAddress(a);
  }

  function dirtyUint() public {
    uint8 a;
    assembly {
      a := not(0)
    }
    emit DirtyUint(a);
  }
}
