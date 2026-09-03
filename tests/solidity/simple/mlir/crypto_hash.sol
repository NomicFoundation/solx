//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "keccak(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x6162636462636465636465666465666765666768666768696768696a68696a6b"
//!                     ],
//!                     "expected": [
//!                         "0x4b50e45e85ca4a0a9c089890faf83098c75b04fe0e0f9c5488effd1643711033"
//!                     ]
//!                 },
//!                 {
//!                     "method": "keccak_calldata(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x6162636462636465636465666465666765666768666768696768696a68696a6b"
//!                     ],
//!                     "expected": [
//!                         "0x4b50e45e85ca4a0a9c089890faf83098c75b04fe0e0f9c5488effd1643711033"
//!                     ]
//!                 },
//!                 {
//!                     "method": "sha(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x993dab3dd91f5c6dc28e17439be475478f5635c92a56e17e82349d3fb2f16619"
//!                     ],
//!                     "expected": [
//!                         "0xfc91f256e276707af00133a3fea5eadc371b4904a429bce31579ef507f942eeb"
//!                     ]
//!                 },
//!                 {
//!                     "method": "ripemd(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000004"
//!                     ],
//!                     "expected": [
//!                         "0x1b0f3c404d12075c68c938f9f60ebea4f74941a0000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "recover(bytes32,uint8,bytes32,bytes32)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad",
//!                         "0x000000000000000000000000000000000000000000000000000000000000001c",
//!                         "0xdebaaa0cddb321b2dcaaf846d39605de7b97e77ba6106587855b9106cb104215",
//!                         "0x61a22d94fa8b8a687ff9c911c844d1c016d1a685a9166858f9c7c1bc85128aca"
//!                     ],
//!                     "expected": [
//!                         "0x0000000000000000000000008743523d96a1b2cbe0c6909653a56da18ed484af"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  function keccak(bytes memory data) public returns (bytes32) {
    return keccak256(data);
  }

  function keccak_calldata(bytes calldata data) public returns (bytes32) {
    return keccak256(data);
  }

  function sha(bytes memory data) public returns (bytes32) {
    return sha256(data);
  }

  function ripemd(bytes memory data) public returns (bytes20) {
    return ripemd160(data);
  }

  function recover(bytes32 hash, uint8 v, bytes32 r, bytes32 s) public returns (address) {
    return ecrecover(hash, v, r, s);
  }
}
