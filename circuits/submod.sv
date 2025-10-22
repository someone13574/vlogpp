`default_nettype none

`include "adder.sv"

module submod (
    input var logic [7:0] a,
    input var logic [7:0] b,
    input var logic c,
    output var logic [8:0] out
);

    logic carry;

    adder adder0 (
        .a  (a[3:0]),
        .b  (b[3:0]),
        .c  (c),
        .out({carry, out[3:0]})
    );

    adder adder1 (
        .a  (a[7:4]),
        .b  (b[7:4]),
        .c  (carry),
        .out(out[8:4])
    );

endmodule
