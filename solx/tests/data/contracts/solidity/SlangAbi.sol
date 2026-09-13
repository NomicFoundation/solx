// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

type Wad is uint256;

interface IAbi {
    function ping(uint256 value) external returns (uint256);
}

library AbiLibrary {
    error LibraryError(uint256 code);

    event LibraryEvent(address account);

    function external_function(uint256 value) external pure returns (uint256) {
        return value;
    }

    function internal_function(uint256 value) internal pure returns (uint256) {
        return value;
    }
}

abstract contract AbiBase {
    function abstract_function(bytes calldata data) external virtual returns (bytes memory);
}

contract SlangAbi {
    enum Choice {
        Left,
        Right
    }

    struct Point {
        uint256 x;
        int128[] ys;
    }

    struct Line {
        Point from;
        Point to;
        Choice choice;
    }

    error Failed(uint256 code, string reason);
    error Empty();

    event Moved(address indexed from, address indexed to, uint256 amount);
    event Traced(Point point) anonymous;

    uint256 public count;
    mapping(address => uint256) public balances;
    bytes32[] public hashes;

    constructor(uint256 initial) payable {
        count = initial;
    }

    receive() external payable {}

    fallback() external {}

    function add(uint256 value) external returns (uint256 total) {
        count += value;
        return count;
    }

    function draw(Line calldata line, address payable target, IAbi callee, Choice choice, Wad wad)
        external
        pure
        returns (Point memory, bool)
    {
        return (line.from, target != address(0) && address(callee) != address(0) && choice == Choice.Left && Wad.unwrap(wad) == 0);
    }

    function lines(Line[] memory input, Point[2] memory pair) external pure returns (Line[] memory) {
        return input;
    }

    function tag(bytes4 selector, string memory label, function(uint256) external callback) external pure returns (bytes4) {
        return selector;
    }
}
