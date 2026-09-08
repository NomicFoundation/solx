//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "recoverInvalid()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "recoverInvalidRawScratch()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "recoverValid()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x8743523d96a1b2cbe0c6909653a56da18ed484af"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  mapping(uint256 => uint256) m;

  function recoverInvalid() public returns (address) {
    m[0xdeadbeef] = 1;
    return ecrecover(
      bytes32(type(uint256).max),
      1,
      bytes32(uint256(2)),
      bytes32(uint256(3))
    );
  }

  function recoverInvalidRawScratch() public view returns (address) {
    assembly {
      mstore(0, not(0))
      mstore(0x20, not(0))
    }
    return ecrecover(
      0x77e5189111eb6557e8a637b27ef8fbb15bc61d61c2f00cc48878f3a296e5e0ca,
      0,
      0x6944c77849b18048f6abe0db8084b0d0d0689cdddb53d2671c36967b58691ad4,
      0xef4f06ba4f78319baafd0424365777241af4dfd3da840471b4b4b087b7750d0d
    );
  }

  function recoverValid() public returns (address) {
    m[0xdeadbeef] = 1;
    return ecrecover(
      0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad,
      28,
      0xdebaaa0cddb321b2dcaaf846d39605de7b97e77ba6106587855b9106cb104215,
      0x61a22d94fa8b8a687ff9c911c844d1c016d1a685a9166858f9c7c1bc85128aca
    );
  }
}
