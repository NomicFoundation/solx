// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IERC165 {
    function supportsInterface(bytes4 id) external view returns (bool);
}

interface ICounter is IERC165 {
    function count() external view returns (uint256);
}

abstract contract Root is ICounter {
    function supportsInterface(bytes4 id) public pure override returns (bool) {
        return id == 0x01ffc9a7;
    }

    function hook(uint256 x) internal virtual returns (uint256);
}
