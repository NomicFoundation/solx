// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*get_sender.*}}
// CHECK-DAG:   sol.caller : !sol.address

// CHECK-DAG: sol.func @{{.*get_value.*}}
// CHECK-DAG:   sol.callvalue : ui256

// CHECK-DAG: sol.func @{{.*get_origin.*}}
// CHECK-DAG:   sol.origin : !sol.address

// CHECK-DAG: sol.func @{{.*get_gasprice.*}}
// CHECK-DAG:   sol.gasprice : ui256

// CHECK-DAG: sol.func @{{.*get_timestamp.*}}
// CHECK-DAG:   sol.timestamp : ui256

// CHECK-DAG: sol.func @{{.*get_number.*}}
// CHECK-DAG:   sol.blocknumber : ui256

// CHECK-DAG: sol.func @{{.*get_coinbase.*}}
// CHECK-DAG:   sol.coinbase : !sol.address

// CHECK-DAG: sol.func @{{.*get_chainid.*}}
// CHECK-DAG:   sol.chainid : ui256

// CHECK-DAG: sol.func @{{.*get_basefee.*}}
// CHECK-DAG:   sol.basefee : ui256

// CHECK-DAG: sol.func @{{.*get_gaslimit.*}}
// CHECK-DAG:   sol.gaslimit : ui256

// CHECK-DAG: sol.func @{{.*get_blobbasefee.*}}
// CHECK-DAG:   sol.blobbasefee : ui256

// CHECK-DAG: sol.func @{{.*get_prevrandao.*}}
// CHECK-DAG:   sol.prevrandao : ui256

// CHECK-DAG: sol.func @{{.*get_difficulty.*}}
// CHECK-DAG:   sol.difficulty : ui256

// CHECK-DAG: sol.func @{{.*get_balance.*}}
// CHECK-DAG:   sol.balance %{{.*}} : !sol.address -> ui256

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

    function get_blobbasefee() public view returns (uint256) {
        return block.blobbasefee;
    }

    function get_prevrandao() public view returns (uint256) {
        return block.prevrandao;
    }

    function get_difficulty() public view returns (uint256) {
        return block.difficulty;
    }

    function get_balance(address a) public view returns (uint256) {
        return a.balance;
    }
}
