// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*local.*}}
// CHECK:   %[[X:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.inline_asm {
// CHECK:     %[[XPTR:.*]] = sol.yul_ptr_cast %[[X]] : !sol.ptr<ui256, Stack> -> !yul.ptr
// CHECK:     yul.load %[[XPTR]] : !yul.ptr -> i256
// CHECK:     %[[XPTR2:.*]] = sol.yul_ptr_cast %[[X]] : !sol.ptr<ui256, Stack> -> !yul.ptr
// CHECK:     yul.store %{{.*}}, %[[XPTR2]] : i256, !yul.ptr

// CHECK: sol.func @{{.*memory_reference.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_ptr_cast %{{.*}} : !sol.ptr<!sol.string<Memory>, Stack> -> !yul.ptr

// CHECK: sol.func @{{.*narrow_locals.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[SEVEN:.*]] = yul.constant 7
// CHECK-NEXT: %[[A:.*]] = sol.yul_ptr_cast %{{.*}} : !sol.ptr<ui8, Stack> -> !yul.ptr
// CHECK-NEXT: yul.store %[[SEVEN]], %[[A]] : i256, !yul.ptr
// CHECK-NEXT: %[[ELEVEN:.*]] = yul.constant 11
// CHECK-NEXT: %[[B:.*]] = sol.yul_ptr_cast %{{.*}} : !sol.ptr<si8, Stack> -> !yul.ptr
// CHECK-NEXT: yul.store %[[ELEVEN]], %[[B]] : i256, !yul.ptr
// CHECK-NEXT: %[[THIRTEEN:.*]] = yul.constant 13
// CHECK-NEXT: %[[CPTR:.*]] = sol.yul_ptr_cast %{{.*}} : !sol.ptr<!sol.fixedbytes<4>, Stack> -> !yul.ptr
// CHECK-NEXT: yul.store %[[THIRTEEN]], %[[CPTR]] : i256, !yul.ptr
// CHECK-NOT:  sol.cast

// CHECK: sol.func @{{.*state_variable.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_state_var_slot @{{.*value.*}}
// CHECK:     sol.yul_state_var_offset @{{.*value.*}}
// CHECK:     sol.yul_state_var_slot @{{.*array.*}}
// CHECK:     sol.yul_state_var_slot @{{.*map.*}}

// CHECK: sol.func @{{.*storage_pointer.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_storage_slot %{{.*}} : !sol.ptr<!sol.array<? x ui256, Storage>, Stack> -> !yul.ptr
// CHECK:     sol.yul_storage_offset %{{.*}} : !sol.ptr<!sol.array<? x ui256, Storage>, Stack>

// CHECK: sol.func @{{.*calldata_reference.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_calldata_offset %{{.*}} -> !yul.ptr
// CHECK:     sol.yul_calldata_length %{{.*}} -> !yul.ptr

// CHECK: sol.func @{{.*function_pointer.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_selector %{{.*}} -> !yul.ptr
// CHECK:     sol.yul_address_of %{{.*}} -> !yul.ptr

// CHECK: sol.func @{{.*writable_suffixes.*}}
// CHECK:   sol.inline_asm {
// CHECK:     %[[SLOT:.*]] = sol.yul_storage_slot %{{.*}} -> !yul.ptr
// CHECK:     yul.store %{{.*}}, %[[SLOT]] : i256, !yul.ptr
// CHECK:     %[[OFF:.*]] = sol.yul_calldata_offset %{{.*}} -> !yul.ptr
// CHECK:     yul.store %{{.*}}, %[[OFF]] : i256, !yul.ptr
// CHECK:     %[[LEN:.*]] = sol.yul_calldata_length %{{.*}} -> !yul.ptr
// CHECK:     yul.store %{{.*}}, %[[LEN]] : i256, !yul.ptr
// CHECK:     %[[SEL:.*]] = sol.yul_selector %{{.*}} -> !yul.ptr
// CHECK:     yul.store %{{.*}}, %[[SEL]] : i256, !yul.ptr
// CHECK:     %[[ADDR:.*]] = sol.yul_address_of %{{.*}} -> !yul.ptr
// CHECK:     yul.store %{{.*}}, %[[ADDR]] : i256, !yul.ptr

// CHECK: sol.func @{{.*constants.*}}
// CHECK:   sol.inline_asm {
// CHECK:     sol.yul_val_cast %{{.*}} -> i256
// CHECK:     sol.yul_val_cast %{{.*}} -> i256
// CHECK:     sol.yul_val_cast %{{.*}} -> i256
// CHECK:     sol.yul_val_cast %{{.*}} : !sol.fixedbytes<32> -> i256

// CHECK: sol.func @{{.*chained_string_constant.*}}
// CHECK:   sol.inline_asm {
// CHECK-NOT: sol.string_lit
// CHECK:     sol.yul_val_cast %{{.*}} : !sol.fixedbytes<32> -> i256
// CHECK:   }

// CHECK: sol.func @{{.*constant_in_yul_function.*}}
// CHECK:   sol.inline_asm {
// CHECK:     yul.func @{{.*folded.*}}
// CHECK:       sol.yul_val_cast %{{.*}} -> i256
// CHECK:       sol.yul_val_cast %{{.*}} -> i256

uint256 constant FILE_LEVEL = 3;

contract C {
    uint256 value;
    uint256[] array;
    mapping(uint256 => uint256) map;
    uint256 constant WIDE = 42;
    int8 constant NARROW = -1;
    bytes32 constant LEFT_ALIGNED = "abc";
    bytes32 constant CHAINED = LEFT_ALIGNED;

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

    function narrow_locals(uint8 a, int8 b, bytes4 c) public pure returns (uint256 r) {
        assembly {
            a := 7
            b := 11
            c := 13
            r := a
        }
    }

    function state_variable() public view returns (uint256 r) {
        assembly {
            let s := value.slot
            let o := value.offset
            r := add(s, o)
            r := add(r, array.slot)
            r := add(r, map.slot)
        }
    }

    function storage_pointer() public view returns (uint256 r) {
        uint256[] storage pointer = array;
        assembly {
            let s := pointer.slot
            let o := pointer.offset
            r := add(s, o)
        }
    }

    function calldata_reference(uint256[] calldata data) public pure returns (uint256 r) {
        assembly {
            let o := data.offset
            let l := data.length
            r := add(o, l)
        }
    }

    function function_pointer(function() external pointer) public pure returns (uint256 r) {
        assembly {
            let s := pointer.selector
            let a := pointer.address
            r := add(s, a)
        }
    }

    function writable_suffixes(
        uint256[] calldata data,
        function() external fn
    ) public view {
        uint256[] storage pointer = array;
        assembly {
            pointer.slot := 3
            data.offset := 64
            data.length := 2
            fn.selector := 0x11223344
            fn.address := 1
        }
    }

    function constants() public pure returns (uint256 r) {
        assembly {
            let w := WIDE
            let n := NARROW
            let f := FILE_LEVEL
            let l := LEFT_ALIGNED
            r := add(add(w, n), add(f, l))
        }
    }

    function chained_string_constant() public pure returns (uint256 r) {
        assembly {
            r := CHAINED
        }
    }

    function constant_in_yul_function() public pure returns (uint256 r) {
        assembly {
            function folded() -> y {
                y := add(WIDE, NARROW)
            }
            r := folded()
        }
    }
}
