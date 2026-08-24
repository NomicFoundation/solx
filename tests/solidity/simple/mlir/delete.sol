//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "uintVar()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetUint()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "uintVar()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "boolVar()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetBool()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "boolVar()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetAddr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetBytes()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetInt()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "s()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetEnum()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "s()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetStaticArrElem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "0",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetStaticArr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetDynArr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetLocalUint()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetDynArrElem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "0",
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetStatic2DArrRow()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetStatic2DArrElem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "0",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetMappingKey()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetNestedMappingKey()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetDynDynArrElem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "0",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetDynDynArrRow()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetDynDynArr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetFixDynArrElem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "0",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetFixDynArrRow()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetFixDynArr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetDynFixArrElem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "0",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetDynFixArrRow()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetDynFixArr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memResetDynDynElem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "0",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memResetDynDynRow()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memResetDynDyn()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memResetFixDynElem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "0",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memResetFixDynRow()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memResetFixDyn()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memResetDynFixElem()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1",
//!                         "0",
//!                         "3",
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memResetDynFixRow()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "memResetDynFix()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0",
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetInternalFp()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetLocalFp()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "resetExtFp()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Semantics tests for the 'delete' operator across value and reference types.

contract Test {
    enum Status { Active, Inactive, Pending }

    uint    public uintVar  = 42;
    bool    public boolVar  = true;
    address public addrVar  = address(0xABCD);
    bytes32 public bytesVar = bytes32(uint(0xDEAD));
    int16   public intVar   = -7;
    Status  public s        = Status.Inactive;

    uint[3]   public staticArr;
    uint[2][2] public static2DArr;
    uint[]    public dynArr;

    uint[][]  public dynDynArr;
    uint[2][] public fixDynArr;
    uint[][2] public dynFixArr;

    mapping(address => uint)                     public balances;
    mapping(address => mapping(address => uint)) public allowances;

    // delete on a uint state variable resets it to 0.
    function resetUint() public {
        delete uintVar;
    }

    // delete on a bool resets it to false.
    function resetBool() public {
        delete boolVar;
    }

    // delete on an address resets it to address(0).
    function resetAddr() public returns (address) {
        delete addrVar;
        return addrVar;
    }

    // delete on bytes32 resets it to 0.
    function resetBytes() public returns (bytes32) {
        delete bytesVar;
        return bytesVar;
    }

    // delete on a signed integer resets it to 0.
    function resetInt() public returns (int16) {
        delete intVar;
        return intVar;
    }

    // delete on a single element of a static storage array zeroes only that slot.
    function resetStaticArrElem() public returns (uint, uint, uint) {
        staticArr[0] = 1; staticArr[1] = 2; staticArr[2] = 3;
        delete staticArr[1];
        return (staticArr[0], staticArr[1], staticArr[2]);
    }

    // delete on a static storage array zeroes all elements.
    function resetStaticArr() public returns (uint, uint, uint) {
        staticArr[0] = 1; staticArr[1] = 2; staticArr[2] = 3;
        delete staticArr;
        return (staticArr[0], staticArr[1], staticArr[2]);
    }

    // delete on a dynamic storage array shrinks it to length 0.
    function resetDynArr() public returns (uint) {
        dynArr.push(10); dynArr.push(20);
        delete dynArr;
        return dynArr.length;
    }

    // delete on an enum resets it to its first member (ordinal 0 = Status.Active).
    function resetEnum() public {
        delete s;
    }

    // delete on a local variable resets it to zero.
    function resetLocalUint() public pure returns (uint) {
        uint v = 99;
        delete v;
        return v;
    }

    // delete on a single element of a dynamic storage array zeroes only that slot.
    function resetDynArrElem() public returns (uint, uint, uint) {
        dynArr.push(1); dynArr.push(2); dynArr.push(3);
        delete dynArr[1];
        return (dynArr[0], dynArr[1], dynArr[2]);
    }

    // delete on a row of a 2D static storage array zeroes only that row.
    function resetStatic2DArrRow() public returns (uint, uint, uint, uint) {
        static2DArr[0][0] = 1; static2DArr[0][1] = 2;
        static2DArr[1][0] = 3; static2DArr[1][1] = 4;
        delete static2DArr[0];
        return (static2DArr[0][0], static2DArr[0][1],
                static2DArr[1][0], static2DArr[1][1]);
    }

    // delete on a single element of a 2D static storage array zeroes only that slot.
    function resetStatic2DArrElem() public returns (uint, uint, uint, uint) {
        static2DArr[0][0] = 1; static2DArr[0][1] = 2;
        static2DArr[1][0] = 3; static2DArr[1][1] = 4;
        delete static2DArr[0][1];
        return (static2DArr[0][0], static2DArr[0][1],
                static2DArr[1][0], static2DArr[1][1]);
    }

    // delete on a mapping entry resets that entry to 0.
    function resetMappingKey() public returns (uint) {
        balances[address(0x1)] = 100;
        delete balances[address(0x1)];
        return balances[address(0x1)];
    }

    // delete on a nested mapping entry resets only that entry.
    function resetNestedMappingKey() public returns (uint) {
        allowances[address(0x1)][address(0x2)] = 100;
        delete allowances[address(0x1)][address(0x2)];
        return allowances[address(0x1)][address(0x2)];
    }

    // uint[][]: delete a single element.
    function resetDynDynArrElem() public returns (uint, uint, uint, uint) {
        delete dynDynArr;
        dynDynArr.push();
        dynDynArr[0].push(1); dynDynArr[0].push(2);
        dynDynArr.push();
        dynDynArr[1].push(3); dynDynArr[1].push(4);
        delete dynDynArr[0][1];
        return (dynDynArr[0][0], dynDynArr[0][1], dynDynArr[1][0], dynDynArr[1][1]);
    }

    // uint[][]: delete a row (inner dynamic array) → length 0.
    function resetDynDynArrRow() public returns (uint) {
        delete dynDynArr;
        dynDynArr.push();
        dynDynArr[0].push(1); dynDynArr[0].push(2);
        dynDynArr.push();
        dynDynArr[1].push(3);
        delete dynDynArr[0];
        return dynDynArr[0].length;
    }

    // uint[][]: delete the whole outer array → length 0.
    function resetDynDynArr() public returns (uint) {
        delete dynDynArr;
        dynDynArr.push();
        dynDynArr[0].push(1);
        dynDynArr.push();
        delete dynDynArr;
        return dynDynArr.length;
    }

    // uint[2][]: delete a single element.
    function resetFixDynArrElem() public returns (uint, uint, uint, uint) {
        delete fixDynArr;
        fixDynArr.push();
        fixDynArr[0][0] = 1; fixDynArr[0][1] = 2;
        fixDynArr.push();
        fixDynArr[1][0] = 3; fixDynArr[1][1] = 4;
        delete fixDynArr[0][1];
        return (fixDynArr[0][0], fixDynArr[0][1], fixDynArr[1][0], fixDynArr[1][1]);
    }

    // uint[2][]: delete a row (fixed-2 inner array) → both elements zero.
    function resetFixDynArrRow() public returns (uint, uint) {
        delete fixDynArr;
        fixDynArr.push();
        fixDynArr[0][0] = 1; fixDynArr[0][1] = 2;
        fixDynArr.push();
        fixDynArr[1][0] = 3;
        delete fixDynArr[0];
        return (fixDynArr[0][0], fixDynArr[0][1]);
    }

    // uint[2][]: delete the whole outer array → length 0.
    function resetFixDynArr() public returns (uint) {
        delete fixDynArr;
        fixDynArr.push();
        fixDynArr[0][0] = 1;
        delete fixDynArr;
        return fixDynArr.length;
    }

    // uint[][2]: delete a single element.
    function resetDynFixArrElem() public returns (uint, uint, uint, uint) {
        delete dynFixArr;
        dynFixArr[0].push(1); dynFixArr[0].push(2);
        dynFixArr[1].push(3); dynFixArr[1].push(4);
        delete dynFixArr[0][1];
        return (dynFixArr[0][0], dynFixArr[0][1], dynFixArr[1][0], dynFixArr[1][1]);
    }

    // uint[][2]: delete a row (inner dynamic array) → length 0.
    function resetDynFixArrRow() public returns (uint) {
        delete dynFixArr;
        dynFixArr[0].push(1); dynFixArr[0].push(2);
        dynFixArr[1].push(3);
        delete dynFixArr[0];
        return dynFixArr[0].length;
    }

    // uint[][2]: delete the whole fixed-2 outer → both inner lengths 0.
    function resetDynFixArr() public returns (uint, uint) {
        delete dynFixArr;
        dynFixArr[0].push(1); dynFixArr[0].push(2);
        dynFixArr[1].push(3);
        delete dynFixArr;
        return (dynFixArr[0].length, dynFixArr[1].length);
    }

    // uint[][] memory: delete a single element.
    function memResetDynDynElem() public pure returns (uint, uint, uint, uint) {
        uint[][] memory arr = new uint[][](2);
        arr[0] = new uint[](2);
        arr[1] = new uint[](2);
        arr[0][0] = 1; arr[0][1] = 2;
        arr[1][0] = 3; arr[1][1] = 4;
        delete arr[0][1];
        return (arr[0][0], arr[0][1], arr[1][0], arr[1][1]);
    }

    // uint[][] memory: delete a row → length 0.
    function memResetDynDynRow() public pure returns (uint) {
        uint[][] memory arr = new uint[][](2);
        arr[0] = new uint[](2);
        arr[1] = new uint[](2);
        arr[0][0] = 1; arr[0][1] = 2;
        arr[1][0] = 3; arr[1][1] = 4;
        delete arr[0];
        return arr[0].length;
    }

    // uint[][] memory: delete the whole outer array → length 0.
    function memResetDynDyn() public pure returns (uint) {
        uint[][] memory arr = new uint[][](2);
        arr[0] = new uint[](2);
        arr[0][0] = 1;
        delete arr;
        return arr.length;
    }

    // uint[2][] memory: delete a single element.
    function memResetFixDynElem() public pure returns (uint, uint, uint, uint) {
        uint[2][] memory arr = new uint[2][](2);
        arr[0][0] = 1; arr[0][1] = 2;
        arr[1][0] = 3; arr[1][1] = 4;
        delete arr[0][1];
        return (arr[0][0], arr[0][1], arr[1][0], arr[1][1]);
    }

    // uint[2][] memory: delete a row → both elements zero.
    function memResetFixDynRow() public pure returns (uint, uint) {
        uint[2][] memory arr = new uint[2][](2);
        arr[0][0] = 1; arr[0][1] = 2;
        arr[1][0] = 3; arr[1][1] = 4;
        delete arr[0];
        return (arr[0][0], arr[0][1]);
    }

    // uint[2][] memory: delete the whole outer array → length 0.
    function memResetFixDyn() public pure returns (uint) {
        uint[2][] memory arr = new uint[2][](2);
        arr[0][0] = 1; arr[1][0] = 2;
        delete arr;
        return arr.length;
    }

    // uint[][2] memory: delete a single element.
    function memResetDynFixElem() public pure returns (uint, uint, uint, uint) {
        uint[][2] memory arr;
        arr[0] = new uint[](2);
        arr[1] = new uint[](2);
        arr[0][0] = 1; arr[0][1] = 2;
        arr[1][0] = 3; arr[1][1] = 4;
        delete arr[0][1];
        return (arr[0][0], arr[0][1], arr[1][0], arr[1][1]);
    }

    // uint[][2] memory: delete a row → length 0.
    function memResetDynFixRow() public pure returns (uint) {
        uint[][2] memory arr;
        arr[0] = new uint[](2);
        arr[1] = new uint[](2);
        arr[0][0] = 1; arr[0][1] = 2;
        arr[1][0] = 3; arr[1][1] = 4;
        delete arr[0];
        return arr[0].length;
    }

    // uint[][2] memory: delete the whole fixed-2 outer → both inner lengths 0.
    function memResetDynFix() public pure returns (uint, uint) {
        uint[][2] memory arr;
        arr[0] = new uint[](2);
        arr[1] = new uint[](2);
        arr[0][0] = 1;
        arr[1][0] = 2;
        delete arr;
        return (arr[0].length, arr[1].length);
    }

    uint public fpCallResult;
    function() internal intFp;
    function() external extFp;

    function fpHelper() internal { fpCallResult += 10; }

    // delete on an internal function pointer state variable resets it to the
    // zero function.  The pointer is called before deletion to verify it was
    // set correctly; it is not called afterwards to avoid a zero-jump panic.
    function resetInternalFp() public returns (uint) {
        fpCallResult = 0;
        intFp = fpHelper;
        intFp();
        delete intFp;
        return fpCallResult;
    }

    function fpLocalHelper() internal pure returns (uint) { return 7; }

    // delete on a local internal function pointer resets it to the zero
    // function.  Same call-before-delete pattern.
    function resetLocalFp() public pure returns (uint) {
        function() internal pure returns (uint) local = fpLocalHelper;
        uint v = local();
        delete local;
        return v;
    }

    function extFpHelper() external {}

    // delete on an external function pointer state variable zeroes the packed
    // (address + selector) storage slot.
    function resetExtFp() public {
        extFp = this.extFpHelper;
        delete extFp;
    }
}
