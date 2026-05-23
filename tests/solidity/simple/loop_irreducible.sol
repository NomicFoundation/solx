//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "trigger",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.30;

contract Test {
    function ext() external returns (int256) { return int256(block.chainid); }

    function trigger() public {
        unchecked {
            int256[] memory idx = new int256[](uint256(1));
            idx[0] = int256(5);
            bool[] memory ai = new bool[](uint256(8));
            ai[2] = true; ai[5] = true;
            int256 t0;
            while (ai[5] || ai[2]) { t0 = this.ext(); break; }
            while (!(ai[5] || ai[2])) { t0 = int256(block.chainid); break; }
            int256 t1;
            for (uint256 i = uint256(0); ((((ai[uint256(idx[0])] || (ai[uint256(int256(2))] || false)) || ((((ai[uint256(int256(5))] || ai[uint256(int256(2))]) && ((ai[uint256(int256(5))] || (ai[uint256(int256(2))] || ((ai[uint256(int256(2))] || ai[uint256(idx[0])]) && true))) && ((ai[uint256(int256(5))] || (ai[uint256(int256(2))] || false)) || false))) && ((ai[uint256(int256(5))] || (ai[uint256(idx[0])] || (ai[uint256(idx[0])] || ai[uint256(int256(2))]))) || (ai[uint256(int256(5))] || (ai[uint256(int256(2))] || (ai[uint256(int256(5))] || ai[uint256(int256(2))]))))) || false)) && ((((ai[uint256(int256(5))] || ai[uint256(int256(2))]) || (false || ((ai[uint256(int256(5))] || ai[uint256(int256(2))]) || false))) || false) && ((ai[uint256(int256(5))] || ai[uint256(int256(2))]) || false))) && (i == uint256(0))); i = (i + uint256(1))) {
                t1 = int256(block.chainid);
            }
            for (uint256 i = uint256(0); ((! (((ai[uint256(idx[0])] || (ai[uint256(int256(2))] || false)) || ((((ai[uint256(int256(5))] || ai[uint256(int256(2))]) && ((ai[uint256(int256(5))] || (ai[uint256(int256(2))] || ((ai[uint256(int256(2))] || ai[uint256(idx[0])]) && true))) && ((ai[uint256(int256(5))] || (ai[uint256(int256(2))] || false)) || false))) && ((ai[uint256(int256(5))] || (ai[uint256(idx[0])] || (ai[uint256(idx[0])] || ai[uint256(int256(2))]))) || (ai[uint256(int256(5))] || (ai[uint256(int256(2))] || (ai[uint256(int256(5))] || ai[uint256(int256(2))]))))) || false)) && ((((ai[uint256(int256(5))] || ai[uint256(int256(2))]) || (false || ((ai[uint256(int256(5))] || ai[uint256(int256(2))]) || false))) || false) && ((ai[uint256(int256(5))] || ai[uint256(int256(2))]) || false)))) && (i == uint256(0))); i = (i + uint256(1))) {
                t1 = int256(block.chainid);
            }
            int256 t2;
            for (uint256 i = uint256(0); ai[5] && (i == uint256(0)); i = (i + uint256(1))) {
                t2 = int256(block.chainid);
            }
            for (uint256 i = uint256(0); !ai[5] && (i == uint256(0)); i = (i + uint256(1))) {
                t2 = int256(block.chainid);
            }
            assert(t0 == (t1 & t2));
        }
    }
}
