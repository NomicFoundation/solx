// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Base {
    function f() external {}
}

interface IDerived is Base {
    function g() external;
}
