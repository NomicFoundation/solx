//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "5"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db0"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(uint256 a) external returns(bytes32 result) {
        assembly {
            mstore(0, a)
            sstore(0, 7)
            sstore(1, 1)
            sstore(2, 2)
            sstore(3, 3)
            sstore(4, 4)
            sstore(5, 5)
            sstore(6, 6)
            sstore(7, 7)
            sstore(8, 8)
            sstore(9, 9)
            sstore(10, 10)
            sstore(11, 11)
            sstore(12, 12)
            sstore(13, 13)
            sstore(14, 14)
            sstore(15, 15)
            sstore(16, 16)
            sstore(17, 17)
            sstore(18, 18)
            sstore(19, 19)
            sstore(20, 20)
            sstore(21, 21)
            sstore(22, 22)
            sstore(23, 23)
            sstore(24, 24)
            sstore(25, 25)
            sstore(26, 26)
            sstore(27, 27)
            sstore(28, 28)
            sstore(29, 29)
            sstore(30, 30)
            sstore(31, 31)
            sstore(32, 32)
            sstore(33, 33)
            sstore(34, 34)
            sstore(35, 35)
            sstore(36, 36)
            sstore(37, 37)
            sstore(38, 38)
            sstore(39, 39)
            sstore(40, 40)
            sstore(41, 41)
            sstore(42, 42)
            sstore(43, 43)
            sstore(44, 44)
            sstore(45, 45)
            sstore(46, 46)
            sstore(47, 47)
            sstore(48, 48)
            sstore(49, 49)
            sstore(50, 50)
            sstore(51, 51)
            sstore(52, 52)
            sstore(53, 53)
            sstore(54, 54)
            sstore(55, 55)
            sstore(56, 56)
            sstore(57, 57)
            sstore(58, 58)
            sstore(59, 59)
            sstore(60, 60)
            sstore(61, 61)
            sstore(62, 62)
            sstore(63, 63)
            sstore(64, 64)
            sstore(65, 65)
            sstore(66, 66)
            sstore(67, 67)
            sstore(68, 68)
            sstore(69, 69)
            sstore(70, 70)
            sstore(71, 71)
            sstore(72, 72)
            sstore(73, 73)
            sstore(74, 74)
            sstore(75, 75)
            sstore(76, 76)
            sstore(77, 77)
            sstore(78, 78)
            sstore(79, 79)
            sstore(80, 80)
            sstore(81, 81)
            sstore(82, 82)
            sstore(83, 83)
            sstore(84, 84)
            sstore(85, 85)
            sstore(86, 86)
            sstore(87, 87)
            sstore(88, 88)
            sstore(89, 89)
            sstore(90, 90)
            sstore(91, 91)
            sstore(92, 92)
            sstore(93, 93)
            sstore(94, 94)
            sstore(95, 95)
            sstore(96, 96)
            sstore(97, 97)
            sstore(98, 98)
            sstore(99, 99)
            result := keccak256(0, 32)
        }
    }
}
