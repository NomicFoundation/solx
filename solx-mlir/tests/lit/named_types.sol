// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}identity_color{{.*}}!sol.enum<2>{{.*}}!sol.enum<2>
// CHECK: sol.func {{.*}}identity_iface{{.*}}!sol.contract<"IFoo{{.*}}">{{.*}}!sol.contract<"IFoo{{.*}}">
// CHECK: sol.func {{.*}}identity_token{{.*}}!sol.contract<"Token{{.*}}">{{.*}}!sol.contract<"Token{{.*}}">
// CHECK: sol.func {{.*}}identity_vault{{.*}}!sol.contract<"Vault{{.*}}", payable>{{.*}}!sol.contract<"Vault{{.*}}", payable>

contract C {
    enum Color { Red, Green, Blue }

    function identity_color(Color c) public pure returns (Color) { return c; }
    function identity_iface(IFoo f) public pure returns (IFoo) { return f; }
    function identity_token(Token t) public pure returns (Token) { return t; }
    function identity_vault(Vault v) public pure returns (Vault) { return v; }
}

interface IFoo { function go() external; }

contract Token {
    function tag() external pure returns (uint8) { return 1; }
}

contract Vault {
    receive() external payable {}
}
