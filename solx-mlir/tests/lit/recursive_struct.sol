// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A struct from which a cycle is reachable through a dynamic array, a mapping or a nested struct
// is an identified type, whose name carries the node id the two frontends number differently. A
// struct whose only path to a cycle runs through a function type stays literal, spelled by its
// members.

// CHECK: sol.contract @{{.*}}A
// CHECK:   sol.state_var @{{.*}}root{{.*}} slot 0 offset 0 : !sol.struct<"Node_{{[0-9]+}}", Storage, (!sol.array<? x !sol.struct<"Node_{{[0-9]+}}", Storage>, Storage>, ui256)>
// CHECK:   sol.state_var @{{.*}}nested{{.*}} slot 2 offset 0 : !sol.struct<"Nested_{{[0-9]+}}", Storage, (ui256, !sol.struct<"Node_{{[0-9]+}}", Storage, {{.*}})>
// CHECK:   sol.state_var @{{.*}}keyed{{.*}} slot 5 offset 0 : !sol.struct<"Keyed_{{[0-9]+}}", Storage, (!sol.mapping<ui256, !sol.struct<"Keyed_{{[0-9]+}}", Storage>>)>
// CHECK:   sol.state_var @{{.*}}left{{.*}} slot 6 offset 0 : !sol.struct<"Left_{{[0-9]+}}", Storage, (!sol.array<? x !sol.struct<"Right_{{[0-9]+}}", Storage, (!sol.struct<"Left_{{[0-9]+}}", Storage>)>, Storage>)>
// CHECK:   sol.state_var @{{.*}}right{{.*}} slot 7 offset 0 : !sol.struct<"Right_{{[0-9]+}}", Storage, (!sol.struct<"Left_{{[0-9]+}}", Storage, (!sol.array<? x !sol.struct<"Right_{{[0-9]+}}", Storage>, Storage>)>)>
// CHECK:   sol.state_var @{{.*}}tagged{{.*}} slot 8 offset 0 : !sol.struct<"Tagged_{{[0-9]+}}", Storage, (!sol.array<? x !sol.struct<"Tagged_{{[0-9]+}}", Storage>, Storage>, !sol.func_ref<(!sol.struct<"Tagged_{{[0-9]+}}", Memory, {{.*}}) -> ()>)>
// CHECK:   sol.state_var @{{.*}}watcher{{.*}} slot 10 offset 0 : !sol.struct<(ui256, !sol.func_ref<(!sol.struct<"Node_{{[0-9]+}}", Memory, {{.*}}) -> ()>), Storage>
// CHECK:   sol.state_var @{{.*}}ping{{.*}} slot 12 offset 0 : !sol.struct<(!sol.func_ref<(!sol.struct<"Pong_{{[0-9]+}}", Memory, {{.*}}) -> ()>), Storage>
// CHECK:   sol.state_var @{{.*}}pong{{.*}} slot 13 offset 0 : !sol.struct<"Pong_{{[0-9]+}}", Storage, (!sol.array<? x !sol.struct<"Pong_{{[0-9]+}}", Storage>, Storage>, !sol.func_ref<(!sol.struct<(!sol.func_ref<(!sol.struct<"Pong_{{[0-9]+}}", Memory, {{.*}}) -> ()>), Memory>) -> ()>)>
// CHECK:   sol.state_var @{{.*}}slots{{.*}} slot 15 offset 0 : !sol.struct<"Slots_{{[0-9]+}}", Storage, (!sol.mapping<ui256, !sol.array<2 x !sol.struct<"Slots_{{[0-9]+}}", Storage>, Storage>>)>

// CHECK:   sol.func @{{.*}}count{{.*}}(%arg0: !sol.struct<"Node_{{[0-9]+}}", CallData, {{.*}}) -> ui256
// CHECK:     sol.length %{{.*}} : !sol.array<? x !sol.struct<"Node_{{[0-9]+}}", CallData, {{.*}}>, CallData>

// CHECK:   sol.func @{{.*}}build{{.*}}(%arg0: ui256) -> ui256
// CHECK:     sol.malloc zero_init : !sol.struct<"Node_{{[0-9]+}}", Memory, {{.*}}>
// CHECK:     sol.malloc %{{.*}} zero_init : ui256 !sol.array<? x !sol.struct<"Node_{{[0-9]+}}", Memory, {{.*}}>, Memory>
// CHECK:     sol.copy %{{.*}}, %{{.*}} : !sol.struct<"Node_{{[0-9]+}}", Memory, {{.*}}>, !sol.struct<"Node_{{[0-9]+}}", Storage, {{.*}}>
// CHECK:     sol.push %{{.*}} : !sol.array<? x !sol.struct<"Node_{{[0-9]+}}", Storage, {{.*}}>, Storage> -> !sol.struct<"Node_{{[0-9]+}}", Storage, {{.*}}>
// CHECK:     sol.pop %{{.*}} : !sol.array<? x !sol.struct<"Node_{{[0-9]+}}", Storage, {{.*}}>, Storage>
// CHECK:     sol.data_loc_cast %{{.*}} : !sol.struct<"Node_{{[0-9]+}}", Storage, {{.*}}>, !sol.struct<"Node_{{[0-9]+}}", Memory, {{.*}}>
// CHECK:     sol.delete %{{.*}} : !sol.struct<"Node_{{[0-9]+}}", Storage, {{.*}}>
// CHECK:     sol.length %{{.*}} : !sol.array<? x !sol.struct<"Node_{{[0-9]+}}", Memory, {{.*}}>, Memory>

// A getter drops the members a recursive struct cannot return, and keeps the rest.
// CHECK: sol.contract @{{.*}}B
// CHECK:   sol.state_var @{{.*}}other{{.*}} slot 0 offset 0 : !sol.struct<"Node_{{[0-9]+}}", Storage, {{.*}}>
// CHECK:     sol.gep %{{.*}}, %{{.*}} : !sol.struct<"Node_{{[0-9]+}}", Storage, {{.*}}>, ui64, !sol.ptr<ui256, Storage>
// CHECK:     sol.return %{{.*}} : ui256

// Two contracts declare a `Node` of their own: the node id in the name keeps the two types apart,
// each with its own body and size.
// CHECK: sol.contract @{{.*}}C
// CHECK:   sol.state_var @{{.*}}own{{.*}} slot 0 offset 0 : !sol.struct<"Node_{{[0-9]+}}", Storage, (!sol.array<? x !sol.struct<"Node_{{[0-9]+}}", Storage>, Storage>)>
// CHECK:   sol.state_var @{{.*}}tail{{.*}} slot 1 offset 0 : ui256
// CHECK: sol.contract @{{.*}}D
// CHECK:   sol.state_var @{{.*}}own{{.*}} slot 0 offset 0 : !sol.struct<"Node_{{[0-9]+}}", Storage, (ui256, !sol.array<? x !sol.struct<"Node_{{[0-9]+}}", Storage>, Storage>)>
// CHECK:   sol.state_var @{{.*}}tail{{.*}} slot 2 offset 0 : ui256

struct Node {
    Node[] kids;
    uint256 v;
}

struct Nested {
    uint256 t;
    Node n;
}

struct Keyed {
    mapping(uint256 => Keyed) m;
}

struct Left {
    Right[] rights;
}

struct Right {
    Left l;
}

struct Tagged {
    Tagged[] kids;
    function(Tagged memory) internal pure tag;
}

struct Watcher {
    uint256 a;
    function(Node memory) internal pure f;
}

struct Ping {
    function(Pong memory) internal pure f;
}

struct Pong {
    Pong[] kids;
    function(Ping memory) internal pure g;
}

struct Slots {
    mapping(uint256 => Slots[2]) m;
}

contract A {
    Node root;
    Nested nested;
    Keyed keyed;
    Left left;
    Right right;
    Tagged tagged;
    Watcher watcher;
    Ping ping;
    Pong pong;
    Slots slots;

    function count(Node calldata n) internal pure returns (uint256) {
        return n.kids.length;
    }

    function build(uint256 n) public returns (uint256) {
        Node memory m;
        m.kids = new Node[](n);
        root = m;
        root.kids.push();
        root.kids.pop();
        Node memory back = root;
        delete root;
        return back.kids.length;
    }
}

contract B {
    Node public other;
}

contract C {
    struct Node {
        Node[] kids;
    }

    Node own;
    uint256 tail;
}

contract D {
    struct Node {
        uint256 v;
        Node[] kids;
    }

    Node own;
    uint256 tail;
}
