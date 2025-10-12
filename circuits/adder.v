module adder (
    input  [3:0] a,
    input  [3:0] b,
    output [4:0] out
);

    assign out = a + b;

endmodule
