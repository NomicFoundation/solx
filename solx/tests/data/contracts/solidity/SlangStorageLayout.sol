// SPDX-License-Identifier: MIT

pragma solidity >=0.8.28;

interface IStorageLayout {
    function balance() external view returns (uint256);
}

library StorageLayoutLibrary {
    function double(uint256 value) internal pure returns (uint256) {
        return value * 2;
    }
}

abstract contract StorageLayoutBase {
    uint8 small;
    address owner;
    bool flag;
    uint256 word;
}

contract StorageLayout is StorageLayoutBase {
    enum Status {
        Idle,
        Active
    }

    struct Position {
        uint128 amount;
        uint64 since;
        Status status;
    }

    uint256 constant LIMIT = 10;
    uint256 immutable created;

    mapping(address => Position) positions;
    uint256[] history;
    uint16[3] recent;
    bytes data;
    string name;
    Status status;
    uint256 transient locked;
    address transient caller;

    constructor() {
        created = block.timestamp;
    }
}
