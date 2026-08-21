//! { "modes": [ "Y", "E", "I" ], "cases": [ {
//!     "name": "spillWithHeap",
//!     "inputs": [
//!         {
//!             "method": "spillWithHeap",
//!             "calldata": [ "7" ]
//!         }
//!     ],
//!     "expected": [
//!         "58321", "8", "9", "10", "11", "12", "13", "14"
//!     ]
//! } ] }

pragma solidity >=0.4.19;

contract Test {
    function g(uint256 x) internal pure returns (uint256) {
        return x * 3 + 1;
    }

    function spillWithHeap(uint256 a) public pure returns (uint256[8] memory buf) {
        uint256 v0 = a + 1;
        uint256 v1 = a + 2;
        uint256 v2 = v1 + v0;
        uint256 v3 = v2 + v1;
        uint256 v4 = v3 + v2;
        uint256 v5 = v4 + v3;
        uint256 v6 = v5 + v4;
        uint256 v7 = v6 + v5;
        uint256 v8 = v7 + v6;
        uint256 v9 = v8 + v7;
        uint256 v10 = v9 + v8;
        uint256 v11 = v10 + v9;
        uint256 v12 = v11 + v10;
        uint256 v13 = v12 + v11;
        uint256 v14 = v13 + v12;
        uint256 v15 = v14 + v13;
        uint256 v16 = v15 + v14;
        uint256 v17 = v16 + v15;

        for (uint256 i = 0; i < 8; i++) {
            buf[i] = a + i;
        }

        uint256 r = g(a);
        r += v0 + v1 + v2 + v3 + v4 + v5 + v6 + v7 + v8 + v9 + v10 + v11 +
             v12 + v13 + v14 + v15 + v16 + v17;
        buf[0] += r;
    }
}
