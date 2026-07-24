//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "#deployer",
//!             "calldata": [
//!                 "1"
//!             ],
//!             "expected": [
//!                 "Test.address"
//!             ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "0" ],
//!             "expected": [ "0x1001" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "1" ],
//!             "expected": [ "0x1003" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "2" ],
//!             "expected": [ "0x1005" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "3" ],
//!             "expected": [ "0x1007" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "4" ],
//!             "expected": [ "0x1009" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "5" ],
//!             "expected": [ "0x100b" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "6" ],
//!             "expected": [ "0x100d" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "7" ],
//!             "expected": [ "0x100f" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "8" ],
//!             "expected": [ "0x1011" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "9" ],
//!             "expected": [ "0x1013" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "10" ],
//!             "expected": [ "0x1015" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "11" ],
//!             "expected": [ "0x1017" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "12" ],
//!             "expected": [ "0x1019" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "13" ],
//!             "expected": [ "0x101b" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "14" ],
//!             "expected": [ "0x101d" ]
//!         },
//!         {
//!             "method": "raw",
//!             "calldata": [ "15" ],
//!             "expected": [ "0x101f" ]
//!         }
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

// Report https://github.com/NomicFoundation/solx/issues/599

pragma solidity >=0.8.0;

contract Test {
    uint256 public immutable v0;
    address public immutable v1;
    uint256 public immutable v2;
    uint256 public immutable v3;
    uint256 public immutable v4;
    address public immutable v5;
    uint256 public immutable v6;
    uint256 public immutable v7;
    uint256 public immutable v8;
    address public immutable v9;
    uint256 public immutable v10;
    uint256 public immutable v11;
    uint256 public immutable v12;
    address public immutable v13;
    uint256 public immutable v14;
    uint256 public immutable v15;

    constructor(uint256 seed) { unchecked {
        v0 = seed*1+0x1000+0;
        v1 = address(uint160(seed*2+0x1000+1));
        v2 = seed*3+0x1000+2;
        v3 = seed*4+0x1000+3;
        v4 = seed*5+0x1000+4;
        v5 = address(uint160(seed*6+0x1000+5));
        v6 = seed*7+0x1000+6;
        v7 = seed*8+0x1000+7;
        v8 = seed*9+0x1000+8;
        v9 = address(uint160(seed*10+0x1000+9));
        v10 = seed*11+0x1000+10;
        v11 = seed*12+0x1000+11;
        v12 = seed*13+0x1000+12;
        v13 = address(uint160(seed*14+0x1000+13));
        v14 = seed*15+0x1000+14;
        v15 = seed*16+0x1000+15;
    } }

    function raw(uint256 i) public view returns (uint256) {
        if (i == 0) return v0;
        if (i == 1) return uint256(uint160(v1));
        if (i == 2) return v2;
        if (i == 3) return v3;
        if (i == 4) return v4;
        if (i == 5) return uint256(uint160(v5));
        if (i == 6) return v6;
        if (i == 7) return v7;
        if (i == 8) return v8;
        if (i == 9) return uint256(uint160(v9));
        if (i == 10) return v10;
        if (i == 11) return v11;
        if (i == 12) return v12;
        if (i == 13) return uint256(uint160(v13));
        if (i == 14) return v14;
        if (i == 15) return v15;
        return 0;
    }
}
