// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A bare Solidity local is its stack slot reinterpreted as a Yul one.
// CHECK: sol.func @{{.*local.*}}
// CHECK:   %[[X:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.inline_asm {
// CHECK:     %[[XPTR:.*]] = sol.yul_ptr_cast %[[X]] : !sol.ptr<ui256, Stack> -> !yul.ptr
// CHECK:     yul.load %[[XPTR]] : !yul.ptr -> i256
// A write reaches the same slot, so the enclosing Solidity code sees it.
// CHECK:     %[[XPTR2:.*]] = sol.yul_ptr_cast %[[X]] : !sol.ptr<ui256, Stack> -> !yul.ptr
// CHECK:     yul.store %{{.*}}, %[[XPTR2]] : i256, !yul.ptr

// A memory reference is bridged the same way: the slot holds the memory pointer.
// CHECK: sol.func @{{.*memory_reference.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_ptr_cast %{{.*}} : !sol.ptr<!sol.string<Memory>, Stack> -> !yul.ptr

// A state variable's `.slot` / `.offset` are compile-time words, not slots.
// CHECK: sol.func @{{.*state_variable.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[SLOT:.*]] = sol.yul_state_var_slot @{{.*value.*}}
// CHECK:     %[[OFF:.*]] = sol.yul_state_var_offset @{{.*value.*}}
// CHECK:     sol.yul_state_var_slot @{{.*array.*}}
// CHECK:     sol.yul_state_var_slot @{{.*map.*}}

// A storage-pointer local carries its slot and offset in the alloca itself.
// CHECK: sol.func @{{.*storage_pointer.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_storage_slot %{{.*}} : !sol.ptr<!sol.array<? x ui256, Storage>, Stack> -> !yul.ptr
// CHECK:     sol.yul_storage_offset %{{.*}} : !sol.ptr<!sol.array<? x ui256, Storage>, Stack>

// A calldata reference carries its offset and length as separate fields.
// CHECK: sol.func @{{.*calldata_reference.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_calldata_offset %{{.*}} -> !yul.ptr
// CHECK:     sol.yul_calldata_length %{{.*}} -> !yul.ptr

// CHECK: sol.func @{{.*function_pointer.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_selector %{{.*}} -> !yul.ptr
// CHECK:     sol.yul_address_of %{{.*}} -> !yul.ptr

// A constant folds in the Sol dialect at its declared type, then crosses over.
// CHECK: sol.func @{{.*constants.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_val_cast %{{.*}} -> i256
// CHECK:     sol.yul_val_cast %{{.*}} -> i256

contract C {
    uint256 value;
    uint256[] array;
    mapping(uint256 => uint256) map;
    uint256 constant WIDE = 42;
    int8 constant NARROW = -1;

    function local(uint256 x) public pure returns (uint256 r) {
        assembly {
            r := x
            x := 1
        }
    }

    function memory_reference(bytes memory data) public pure returns (uint256 r) {
        assembly {
            r := data
        }
    }

    function state_variable() public view returns (uint256 r) {
        assembly {
            r := add(value.slot, value.offset)
            r := add(r, array.slot)
            r := add(r, map.slot)
        }
    }

    function storage_pointer() public view returns (uint256 r) {
        uint256[] storage pointer = array;
        assembly {
            r := add(pointer.slot, pointer.offset)
        }
    }

    function calldata_reference(uint256[] calldata data) public pure returns (uint256 r) {
        assembly {
            r := add(data.offset, data.length)
        }
    }

    function function_pointer(function() external pointer) public pure returns (uint256 r) {
        assembly {
            r := add(pointer.selector, pointer.address)
        }
    }

    function constants() public pure returns (uint256 r) {
        assembly {
            r := add(WIDE, NARROW)
        }
    }
}
