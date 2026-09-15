//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "set_1s(uint8[5])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "40",
//!                         "50"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_1s()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_1s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_1s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "20"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_1s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "30"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_1s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "40"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_1s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "50"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_1d(uint8[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "5",
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "5"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_1d()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_1d(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_1d(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_2ss()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_2ss()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_2ss(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2ss(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2ss(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2ss(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_2ds()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_2ds()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_2ds(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2ds(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "6"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2ds(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2ds(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "8"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_2sd()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_2sd()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_2sd(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "9"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2sd(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2sd(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2sd(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "12"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_2dd()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_2dd()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_2dd(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "13"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2dd(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "14"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2dd(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "15"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_2dd(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "16"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_64s(uint8[5])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "10",
//!                         "20",
//!                         "30",
//!                         "40",
//!                         "50"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_64s()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_64s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "20"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "30"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "3"
//!                     ],
//!                     "expected": [
//!                         "40"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64s(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "50"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_64d(uint8[])",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "5",
//!                         "1",
//!                         "2",
//!                         "3",
//!                         "4",
//!                         "5"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_64d()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_64d(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64d(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64d(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "4"
//!                     ],
//!                     "expected": [
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_64ss()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_64ss()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_64ss(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64ss(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "20"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64ss(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "30"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64ss(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "40"
//!                     ]
//!                 },
//!                 {
//!                     "method": "set_64ds()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "copy_64ds()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "get_64ds(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "50"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64ds(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "60"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64ds(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "70"
//!                     ]
//!                 },
//!                 {
//!                     "method": "get_64ds(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "80"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;


contract Test {


    uint8[5]    s8s;
    uint160[5]  s160s;

    uint8[]     d8;
    uint160[]   d160;

    uint8[2][2]   s8_ss;
    uint160[2][2] s160_ss;

    uint8[2][]    s8_ds;
    uint160[2][]  s160_ds;

    uint8[][2]    s8_sd;
    uint160[][2]  s160_sd;

    uint8[][]     s8_dd;
    uint160[][]   s160_dd;

    uint8[5]   s8_64s;
    uint64[5]  s64_8s;

    uint8[]    d8_64;
    uint64[]   d64_8;

    uint8[2][2]  s8_64_ss;
    uint64[2][2] s64_8_ss;

    uint8[2][]   s8_64_ds;
    uint64[2][]  s64_8_ds;


    function set_1s(uint8[5] calldata src) public { s8s = src; }
    function copy_1s() public { s160s = s8s; }
    function get_1s(uint256 i) public view returns (uint160) { return s160s[i]; }


    function set_1d(uint8[] calldata src) public { d8 = src; }
    function copy_1d() public { d160 = d8; }
    function get_1d(uint256 i) public view returns (uint160) { return d160[i]; }


    function set_2ss() public {
        s8_ss[0][0] = 1; s8_ss[0][1] = 2;
        s8_ss[1][0] = 3; s8_ss[1][1] = 4;
    }
    function copy_2ss() public { s160_ss = s8_ss; }
    function get_2ss(uint256 i, uint256 j) public view returns (uint160) { return s160_ss[i][j]; }


    function set_2ds() public {
        delete s8_ds;
        s8_ds.push();
        s8_ds.push();
        s8_ds[0][0] = 5; s8_ds[0][1] = 6;
        s8_ds[1][0] = 7; s8_ds[1][1] = 8;
    }
    function copy_2ds() public { s160_ds = s8_ds; }
    function get_2ds(uint256 i, uint256 j) public view returns (uint160) { return s160_ds[i][j]; }


    function set_2sd() public {
        delete s8_sd[0];
        delete s8_sd[1];
        s8_sd[0].push(9);  s8_sd[0].push(10);
        s8_sd[1].push(11); s8_sd[1].push(12);
    }
    function copy_2sd() public { s160_sd = s8_sd; }
    function get_2sd(uint256 i, uint256 j) public view returns (uint160) { return s160_sd[i][j]; }


    function set_2dd() public {
        delete s8_dd;
        s8_dd.push();
        s8_dd.push();
        s8_dd[0].push(13); s8_dd[0].push(14);
        s8_dd[1].push(15); s8_dd[1].push(16);
    }
    function copy_2dd() public { s160_dd = s8_dd; }
    function get_2dd(uint256 i, uint256 j) public view returns (uint160) { return s160_dd[i][j]; }


    function set_64s(uint8[5] calldata src) public {
        s8_64s = src;
    }
    function copy_64s() public {
        s64_8s = s8_64s;
    }
    function get_64s(uint256 i) public view returns (uint64) {
        return s64_8s[i];
    }

    function set_64d(uint8[] calldata src) public {
        d8_64 = src;
    }
    function copy_64d() public {
        d64_8 = d8_64;
    }
    function get_64d(uint256 i) public view returns (uint64) {
        return d64_8[i];
    }

    function set_64ss() public {
        s8_64_ss[0][0] = 10; s8_64_ss[0][1] = 20;
        s8_64_ss[1][0] = 30; s8_64_ss[1][1] = 40;
    }
    function copy_64ss() public {
        s64_8_ss = s8_64_ss;
    }
    function get_64ss(uint256 i, uint256 j) public view returns (uint64) {
        return s64_8_ss[i][j];
    }

    function set_64ds() public {
        delete s8_64_ds;
        s8_64_ds.push();
        s8_64_ds.push();
        s8_64_ds[0][0] = 50; s8_64_ds[0][1] = 60;
        s8_64_ds[1][0] = 70; s8_64_ds[1][1] = 80;
    }
    function copy_64ds() public {
        s64_8_ds = s8_64_ds;
    }
    function get_64ds(uint256 i, uint256 j) public view returns (uint64) {
        return s64_8_ds[i][j];
    }
}
