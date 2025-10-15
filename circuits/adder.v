module adder (
    input  [3:0] a,
    input  [3:0] b,
    output [4:0] c,
    output [3:0] d
);

    assign c = a + b;

    wire [2:0] a_in;
    assign a_in = {c[2], a[2], b[2]};

    submod submod (
        .a  (a_in),
        .b  (b[2:0]),
        .out(d)
    );

endmodule


module submod (
    input  [2:0] a,
    input  [2:0] b,
    output [3:0] out
);
    assign out = a + b;
endmodule
