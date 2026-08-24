//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "dirtyStart(uint256[],uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x60",
//!                         "0x100000000000000000000000000000000000000000000000000000000000001",
//!                         "3",
//!                         "4",
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "40"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "dirtyEnd(uint256[],uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x60",
//!                         "1",
//!                         "0x100000000000000000000000000000000000000000000000000000000000003",
//!                         "4",
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "40"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function dirtyStart(uint256[] calldata arr, uint256 dirty, uint256 end)
        external
        pure
        returns (uint256)
    {
        uint8 start;
        assembly { start := dirty }

        uint256[] calldata s = arr[start:end];
        return s.length;
    }

    function dirtyEnd(uint256[] calldata arr, uint256 start, uint256 dirty)
        external
        pure
        returns (uint256)
    {
        uint8 end;
        assembly { end := dirty }

        uint256[] calldata s = arr[start:end];
        return s.length;
    }
}
