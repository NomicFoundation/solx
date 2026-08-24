//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "#deployer",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "Test.address"
//!                         ],
//!                         "events": [
//!                             {
//!                                 "topics": [
//!                                     "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
//!                                     "0x00",
//!                                     "0x1212121212121212121212121212120000000012"
//!                                 ],
//!                                 "values": [
//!                                     "0x14"
//!                                 ]
//!                             }
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "totalSupply()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "20"
//!                     ]
//!                 },
//!                 {
//!                     "method": "transfer(address,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2",
//!                         "5"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "1"
//!                         ],
//!                         "events": [
//!                             {
//!                                 "topics": [
//!                                     "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
//!                                     "0x1212121212121212121212121212120000000012",
//!                                     "0x02"
//!                                 ],
//!                                 "values": [
//!                                     "0x05"
//!                                 ]
//!                             }
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "decreaseAllowance(address,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2",
//!                         "0"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "1"
//!                         ],
//!                         "events": [
//!                             {
//!                                 "topics": [
//!                                     "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925",
//!                                     "0x1212121212121212121212121212120000000012",
//!                                     "0x02"
//!                                 ],
//!                                 "values": [
//!                                     "0x00"
//!                                 ]
//!                             }
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "decreaseAllowance(address,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2",
//!                         "1"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "transfer(address,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2",
//!                         "14"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "1"
//!                         ],
//!                         "events": [
//!                             {
//!                                 "topics": [
//!                                     "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
//!                                     "0x1212121212121212121212121212120000000012",
//!                                     "0x02"
//!                                 ],
//!                                 "values": [
//!                                     "0x0e"
//!                                 ]
//!                             }
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "transfer(address,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2",
//!                         "2"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.0 <0.9.0;

contract Test {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    mapping (address => uint256) private _balances;
    mapping (address => mapping (address => uint256)) private _allowances;
    uint256 private _totalSupply;

    constructor() {
        _mint(msg.sender, 20);
    }

    function totalSupply() public view returns (uint256) {
        return _totalSupply;
    }

    function balanceOf(address owner) public view returns (uint256) {
        return _balances[owner];
    }

    function allowance(address owner, address spender) public view returns (uint256) {
        return _allowances[owner][spender];
    }

    function transfer(address to, uint256 value) public returns (bool) {
        _transfer(msg.sender, to, value);
        return true;
    }

    function approve(address spender, uint256 value) public returns (bool) {
        _approve(msg.sender, spender, value);
        return true;
    }

    function transferFrom(address from, address to, uint256 value) public returns (bool) {
        _transfer(from, to, value);
        // The subtraction here will revert on overflow.
        _approve(from, msg.sender, _allowances[from][msg.sender] - value);
        return true;
    }

    function increaseAllowance(address spender, uint256 addedValue) public returns (bool) {
        // The addition here will revert on overflow.
        _approve(msg.sender, spender, _allowances[msg.sender][spender] + addedValue);
        return true;
    }

    function decreaseAllowance(address spender, uint256 subtractedValue) public returns (bool) {
        // The subtraction here will revert on overflow.
        _approve(msg.sender, spender, _allowances[msg.sender][spender] - subtractedValue);
        return true;
    }

    function _transfer(address from, address to, uint256 value) internal {
        require(to != address(0), "ERC20: transfer to the zero address");

        // The subtraction and addition here will revert on overflow.
        _balances[from] = _balances[from] - value;
        _balances[to] = _balances[to] + value;
        emit Transfer(from, to, value);
    }

    function _mint(address account, uint256 value) internal {
        require(account != address(0), "ERC20: mint to the zero address");

        // The additions here will revert on overflow.
        _totalSupply = _totalSupply + value;
        _balances[account] = _balances[account] + value;
        emit Transfer(address(0), account, value);
    }

    function _burn(address account, uint256 value) internal {
        require(account != address(0), "ERC20: burn from the zero address");

        // The subtractions here will revert on overflow.
        _totalSupply = _totalSupply - value;
        _balances[account] = _balances[account] - value;
        emit Transfer(account, address(0), value);
    }

    function _approve(address owner, address spender, uint256 value) internal {
        require(owner != address(0), "ERC20: approve from the zero address");
        require(spender != address(0), "ERC20: approve to the zero address");

        _allowances[owner][spender] = value;
        emit Approval(owner, spender, value);
    }

    function _burnFrom(address account, uint256 value) internal {
        _burn(account, value);
        _approve(account, msg.sender, _allowances[account][msg.sender] - value);
    }
}
