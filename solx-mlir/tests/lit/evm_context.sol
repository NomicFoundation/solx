// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"get_sender()"
// CHECK:   sol.caller : !sol.address

// CHECK: sol.func @"get_value()"
// CHECK:   sol.callvalue : ui256

// CHECK: sol.func @"get_origin()"
// CHECK:   sol.origin : !sol.address

// CHECK: sol.func @"get_gasprice()"
// CHECK:   sol.gasprice : ui256

// CHECK: sol.func @"get_timestamp()"
// CHECK:   sol.timestamp : ui256

// CHECK: sol.func @"get_number()"
// CHECK:   sol.blocknumber : ui256

// CHECK: sol.func @"get_coinbase()"
// CHECK:   sol.coinbase : !sol.address

// CHECK: sol.func @"get_chainid()"
// CHECK:   sol.chainid : ui256

// CHECK: sol.func @"get_basefee()"
// CHECK:   sol.basefee : ui256

// CHECK: sol.func @"get_gaslimit()"
// CHECK:   sol.gaslimit : ui256

contract C {
    function get_sender() public view returns (address) {
        return msg.sender;
    }

    function get_value() public payable returns (uint256) {
        return msg.value;
    }

    function get_origin() public view returns (address) {
        return tx.origin;
    }

    function get_gasprice() public view returns (uint256) {
        return tx.gasprice;
    }

    function get_timestamp() public view returns (uint256) {
        return block.timestamp;
    }

    function get_number() public view returns (uint256) {
        return block.number;
    }

    function get_coinbase() public view returns (address) {
        return block.coinbase;
    }

    function get_chainid() public view returns (uint256) {
        return block.chainid;
    }

    function get_basefee() public view returns (uint256) {
        return block.basefee;
    }

    function get_gaslimit() public view returns (uint256) {
        return block.gaslimit;
    }
}
