// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #{{.*}}Constructor

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #{{.*}}Receive, state_mutability = #{{.*}}Payable

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #{{.*}}Fallback{{.*}}state_mutability = #{{.*}}Payable

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Fallback{{.*}}state_mutability = #NonPayable
// CHECK:   sol.get_calldata : !sol.string<CallData>
// CHECK:   sol.length %{{.*}} : !sol.string<CallData>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Fallback{{.*}}state_mutability = #NonPayable
// CHECK:   sol.sig : !sol.fixedbytes<4>
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, !sol.ptr<!sol.fixedbytes<4>, Storage>

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Fallback{{.*}}state_mutability = #Payable
// CHECK:   %[[VALUE:.*]] = sol.callvalue : ui256
// CHECK:   sol.store %[[VALUE]], %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*}}() attributes {{.*}}kind = #Receive{{.*}}state_mutability = #Payable
// CHECK:   %[[VALUE:.*]] = sol.callvalue : ui256
// CHECK:   sol.store %[[VALUE]], %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract AllKinds {
    uint256 x;

    constructor(uint256 val) {
        x = val;
    }

    receive() external payable {}

    fallback() external payable {}

    function get() public view returns (uint256) {
        return x;
    }
}

contract MsgDataLength {
    uint256 public lastLength;

    fallback() external {
        lastLength = msg.data.length;
    }
}

contract MsgSig {
    bytes4 public lastSignature;

    fallback() external {
        lastSignature = msg.sig;
    }
}

contract PayableFallback {
    uint256 public received;

    fallback() external payable {
        received = msg.value;
    }
}

contract PayableReceive {
    uint256 public total;

    receive() external payable {
        total = msg.value;
    }
}
