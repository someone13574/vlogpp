module adder (
    input a,
    input b,
    input c,
    output [1:0] out
);

    assign out[0] = (a ^ b) & c;
    assign out[1] = (a ^ b) | c;

endmodule
