module adder (
    input [3:0] a,
    input [3:0] b,
    input c,
    output [4:0] out
);

    assign out = a + b + c;

endmodule
