// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

interface I {
    function f() external view returns (bool);
}

abstract contract A is I {}
