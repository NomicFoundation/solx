// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK: sol.ext_call "{{.*}}"() at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = () -> ui256, static_call} : !sol.address, () -> (i1, ui256)

// CHECK: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK: sol.ext_call "{{.*}}"(%{{.*}}) at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK: sol.ext_call "{{.*}}"(%{{.*}}) at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call} : !sol.address, (ui256) -> (i1, ui256)

// CHECK: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK: %{{.*}}, %[[R:.*]]:2 = sol.ext_call "{{.*}}"() at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = () -> (ui256, ui256), static_call} : !sol.address, () -> (i1, ui256, ui256)
// CHECK: sol.return %[[R]]#0, %[[R]]#1 : ui256, ui256

// CHECK: sol.ext_call "{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call} : !sol.address, (ui256) -> (i1, ui256)
// CHECK: sol.ext_call "{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call} : !sol.address, (ui256) -> (i1, ui256)

contract Elementary {
    uint256 public x;

    function g() external returns (uint256) {
        return this.x();
    }
}

contract Mapping {
    mapping(uint256 => uint256) public m;

    function g(uint256 k) external returns (uint256) {
        return this.m(k);
    }
}

contract Sequence {
    uint256[] public array;

    function g(uint256 i) external returns (uint256) {
        return this.array(i);
    }
}

contract Struct {
    struct S {
        uint256 a;
        uint256 b;
    }

    S public s;

    function g() external returns (uint256, uint256) {
        return this.s();
    }
}

contract Token {
    mapping(uint256 => uint256) public m;
    uint256[] public array;
}

contract Viewer {
    function readArray(Token o, uint256 index) external view returns (uint256) {
        return o.array(index);
    }

    function readMapping(Token o, uint256 key) external view returns (uint256) {
        return o.m(key);
    }
}
