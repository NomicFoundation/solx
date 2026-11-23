// RUN: slang --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*fromCalldata.*}}(
// CHECK:   %[[CALLDATA:.*]] = sol.data_loc_cast %{{.*}} : !sol.string<CallData>, !sol.string<Memory>
// CHECK:   sol.erc7201"(%[[CALLDATA]]) : (!sol.string<Memory>) -> ui256
// CHECK: sol.func @{{.*fromStorage.*}}(
// CHECK:   %[[STORAGE:.*]] = sol.data_loc_cast %{{.*}} : !sol.string<Storage>, !sol.string<Memory>
// CHECK:   sol.erc7201"(%[[STORAGE]]) : (!sol.string<Memory>) -> ui256

contract C {
    string name;

    function fromCalldata(string calldata namespace) external pure returns (uint256) { return erc7201(namespace); }

    function fromStorage() public view returns (uint256) { return erc7201(name); }
}
