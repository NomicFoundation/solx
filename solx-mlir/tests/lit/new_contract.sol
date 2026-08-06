// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc print-init evaluates a creation's constructor arguments before its options, unlike legacy,
// and casts a string-typed salt straight to integer, failing module verification, so this is
// solx-only.

// CHECK: sol.func @{{.*plain.*}}
// CHECK:   %[[FIRST:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[SECOND:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[VALUE:.*]] = sol.constant 0 : ui256
// CHECK:   sol.new "{{.*}}:Child" value = %[[VALUE]] ctor(%[[FIRST]], %[[SECOND]] : ui256, ui256) : !sol.contract<"{{.*}}:Child">

// CHECK: sol.func @{{.*named.*}}
// CHECK:   %[[FIRST:.*]] = sol.cast %c11_ui8
// CHECK:   %[[SECOND:.*]] = sol.cast %c99_ui8
// CHECK:   sol.new "{{.*}}:Child" value = %{{.*}} ctor(%[[FIRST]], %[[SECOND]] : ui256, ui256) : !sol.contract<"{{.*}}:Child">

// CHECK: sol.func @{{.*value_and_salt.*}}
// CHECK:   %[[VALUE:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[SALT:.*]] = sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to ui256
// CHECK:   %[[FIRST:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   %[[SECOND:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.new "{{.*}}:Child" value = %[[VALUE]] salt = %[[SALT]] ctor(%[[FIRST]], %[[SECOND]] : ui256, ui256) : !sol.contract<"{{.*}}:Child">

// CHECK: sol.func @{{.*literal_salt.*}}
// CHECK:   %[[LITERAL:.*]] = sol.constant 452312848583266388373324160190187140051835877600158453279131187530910662656 : ui256
// CHECK:   %[[BYTES:.*]] = sol.bytes_cast %[[LITERAL]] : ui256 to !sol.fixedbytes<32>
// CHECK:   %[[SALT:.*]] = sol.bytes_cast %[[BYTES]] : !sol.fixedbytes<32> to ui256
// CHECK:   sol.new "{{.*}}:Empty" value = %{{.*}} salt = %[[SALT]] ctor() : !sol.contract<"{{.*}}:Empty">

// CHECK: sol.func @{{.*salt_only.*}}
// CHECK:   %[[SALT:.*]] = sol.bytes_cast %{{.*}} : !sol.fixedbytes<32> to ui256
// CHECK:   %[[VALUE:.*]] = sol.constant 0 : ui256
// CHECK:   sol.new "{{.*}}:Empty" value = %[[VALUE]] salt = %[[SALT]] ctor() : !sol.contract<"{{.*}}:Empty">

// CHECK: sol.func @{{.*value_only.*}}
// CHECK:   %[[VALUE:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.new "{{.*}}:Empty" value = %[[VALUE]] ctor() : !sol.contract<"{{.*}}:Empty">

// CHECK: sol.func @{{.*reference_argument.*}}
// CHECK:   %[[SLOT:.*]] = sol.addr_of @{{.*}} : !sol.array<? x ui256, Storage>
// CHECK:   sol.new "{{.*}}:Holder" value = %{{.*}} ctor(%[[SLOT]] : !sol.array<? x ui256, Storage>) : !sol.contract<"{{.*}}:Holder">

// CHECK: sol.func @{{.*parenthesized.*}}
// CHECK:   sol.new "{{.*}}:Child" value = %{{.*}} ctor(%{{.*}}, %{{.*}} : ui256, ui256) : !sol.contract<"{{.*}}:Child">

contract C {
    uint256[] stored;

    function plain() public returns (Child) {
        return new Child(7, 8);
    }

    function named() public returns (Child) {
        return new Child({b: 99, a: 11});
    }

    function value_and_salt(uint256 v, bytes32 s) public returns (Child) {
        return new Child{value: v, salt: s}(9, 10);
    }

    function literal_salt() public returns (Empty) {
        return new Empty{salt: hex"01"}();
    }

    function salt_only(bytes32 s) public returns (Empty) {
        return new Empty{salt: s}();
    }

    function value_only(uint256 v) public returns (Empty) {
        return new Empty{value: v}();
    }

    function reference_argument() public returns (Holder) {
        return new Holder(stored);
    }

    function parenthesized() public returns (Child) {
        return (new Child)(12, 13);
    }
}

contract Child {
    uint256 sum;

    constructor(uint256 a, uint256 b) payable {
        sum = a + b;
    }
}

contract Empty {
    constructor() payable {}
}

contract Holder {
    uint256 length;

    constructor(uint256[] memory values) {
        length = values.length;
    }
}
