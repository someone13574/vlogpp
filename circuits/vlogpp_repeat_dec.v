module vlogpp_repeat_dec (
    input [WIDTH - 1:0] x,
    output exit,
    output [WIDTH - 1:0] next
);

    parameter integer WIDTH = 8;

    assign exit = x == 0;
    assign next = x - 1;

endmodule
