// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.contract @{{.*}}A
// CHECK:   sol.state_var @{{.*}}root{{.*}} slot 0 offset 0 : !sol.struct<"Node_[[NODE:[0-9]+]]", Storage, (!sol.array<? x !sol.struct<"Node_[[NODE]]", Storage>, Storage>, ui256)>
// CHECK:   sol.state_var @{{.*}}nested{{.*}} slot 2 offset 0 : !sol.struct<"Nested_{{[0-9]+}}", Storage, (ui256, !sol.struct<"Node_[[NODE]]", Storage, (!sol.array<? x !sol.struct<"Node_[[NODE]]", Storage>, Storage>, ui256)>)>
// CHECK:   sol.state_var @{{.*}}keyed{{.*}} slot 5 offset 0 : !sol.struct<"Keyed_[[KEYED:[0-9]+]]", Storage, (!sol.mapping<ui256, !sol.struct<"Keyed_[[KEYED]]", Storage>>)>
// CHECK:   sol.state_var @{{.*}}left{{.*}} slot 6 offset 0 : !sol.struct<"Left_[[LEFT:[0-9]+]]", Storage, (!sol.array<? x !sol.struct<"Right_[[RIGHT:[0-9]+]]", Storage, (!sol.struct<"Left_[[LEFT]]", Storage>)>, Storage>)>
// CHECK:   sol.state_var @{{.*}}right{{.*}} slot 7 offset 0 : !sol.struct<"Right_[[RIGHT]]", Storage, (!sol.struct<"Left_[[LEFT]]", Storage, (!sol.array<? x !sol.struct<"Right_[[RIGHT]]", Storage>, Storage>)>)>
// CHECK:   sol.state_var @{{.*}}tagged{{.*}} slot 8 offset 0 : !sol.struct<"Tagged_[[TAGGED:[0-9]+]]", Storage, (!sol.array<? x !sol.struct<"Tagged_[[TAGGED]]", Storage>, Storage>, !sol.func_ref<(!sol.struct<"Tagged_[[TAGGED]]", Memory, (!sol.array<? x !sol.struct<"Tagged_[[TAGGED]]", Memory>, Memory>, !sol.func_ref<(!sol.struct<"Tagged_[[TAGGED]]", Memory>) -> ()>)>) -> ()>)>
// CHECK:   sol.state_var @{{.*}}watcher{{.*}} slot 10 offset 0 : !sol.struct<(ui256, !sol.func_ref<(!sol.struct<"Node_[[NODE]]", Memory, (!sol.array<? x !sol.struct<"Node_[[NODE]]", Memory>, Memory>, ui256)>) -> ()>), Storage>
// CHECK:   sol.state_var @{{.*}}ping{{.*}} slot 12 offset 0 : !sol.struct<(!sol.func_ref<(!sol.struct<"Pong_[[PONG:[0-9]+]]", Memory, (!sol.array<? x !sol.struct<"Pong_[[PONG]]", Memory>, Memory>, !sol.func_ref<(!sol.struct<(!sol.func_ref<(!sol.struct<"Pong_[[PONG]]", Memory>) -> ()>), Memory>) -> ()>)>) -> ()>), Storage>
// CHECK:   sol.state_var @{{.*}}pong{{.*}} slot 13 offset 0 : !sol.struct<"Pong_[[PONG]]", Storage, (!sol.array<? x !sol.struct<"Pong_[[PONG]]", Storage>, Storage>, !sol.func_ref<(!sol.struct<(!sol.func_ref<(!sol.struct<"Pong_[[PONG]]", Memory, (!sol.array<? x !sol.struct<"Pong_[[PONG]]", Memory>, Memory>, !sol.func_ref<(!sol.struct<(!sol.func_ref<(!sol.struct<"Pong_[[PONG]]", Memory>) -> ()>), Memory>) -> ()>)>) -> ()>), Memory>) -> ()>)>
// CHECK:   sol.state_var @{{.*}}slots{{.*}} slot 15 offset 0 : !sol.struct<"Slots_[[SLOTS:[0-9]+]]", Storage, (!sol.mapping<ui256, !sol.array<2 x !sol.struct<"Slots_[[SLOTS]]", Storage>, Storage>>)>

// CHECK:   sol.func @{{.*}}count{{.*}}(%arg0: !sol.struct<"Node_[[NODE]]", CallData, (!sol.array<? x !sol.struct<"Node_[[NODE]]", CallData>, CallData>, ui256)>) -> ui256
// CHECK:     sol.length %{{.*}} : !sol.array<? x !sol.struct<"Node_[[NODE]]", CallData, {{.*}}>, CallData>

// CHECK:   sol.func @{{.*}}build{{.*}}(%arg0: ui256) -> ui256
// CHECK:     sol.malloc zero_init : !sol.struct<"Node_[[NODE]]", Memory, {{.*}}>
// CHECK:     sol.malloc %{{.*}} zero_init : ui256 !sol.array<? x !sol.struct<"Node_[[NODE]]", Memory, {{.*}}>, Memory>
// CHECK:     sol.copy %{{.*}}, %{{.*}} : !sol.struct<"Node_[[NODE]]", Memory, {{.*}}>, !sol.struct<"Node_[[NODE]]", Storage, {{.*}}>
// CHECK:     sol.push %{{.*}} : !sol.array<? x !sol.struct<"Node_[[NODE]]", Storage, {{.*}}>, Storage> -> !sol.struct<"Node_[[NODE]]", Storage, {{.*}}>
// CHECK:     sol.pop %{{.*}} : !sol.array<? x !sol.struct<"Node_[[NODE]]", Storage, {{.*}}>, Storage>
// CHECK:     sol.data_loc_cast %{{.*}} : !sol.struct<"Node_[[NODE]]", Storage, {{.*}}>, !sol.struct<"Node_[[NODE]]", Memory, {{.*}}>
// CHECK:     sol.delete %{{.*}} : !sol.struct<"Node_[[NODE]]", Storage, {{.*}}>
// CHECK:     sol.length %{{.*}} : !sol.array<? x !sol.struct<"Node_[[NODE]]", Memory, {{.*}}>, Memory>

// CHECK: sol.contract @{{.*}}B
// CHECK:   sol.state_var @{{.*}}other{{.*}} slot 0 offset 0 : !sol.struct<"Node_[[NODE]]", Storage, {{.*}}>
// CHECK:     sol.gep %{{.*}}, %{{.*}} : !sol.struct<"Node_[[NODE]]", Storage, {{.*}}>, ui64, !sol.ptr<ui256, Storage>
// CHECK:     sol.return %{{.*}} : ui256

// CHECK: sol.contract @{{.*}}C
// CHECK:   sol.state_var @{{.*}}own{{.*}} slot 0 offset 0 : !sol.struct<"Node_[[C_NODE:[0-9]+]]", Storage, (!sol.array<? x !sol.struct<"Node_[[C_NODE]]", Storage>, Storage>)>
// CHECK:   sol.state_var @{{.*}}tail{{.*}} slot 1 offset 0 : ui256
// CHECK: sol.contract @{{.*}}D
// CHECK:   sol.state_var @{{.*}}own{{.*}} slot 0 offset 0 : !sol.struct<"Node_[[D_NODE:[0-9]+]]", Storage, (ui256, !sol.array<? x !sol.struct<"Node_[[D_NODE]]", Storage>, Storage>)>
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
