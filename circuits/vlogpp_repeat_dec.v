module vlogpp_repeat_dec (
    input exit_i,
    input [WIDTH - 1:0] x,
    output exit,
    output [WIDTH - 1:0] next
);

    parameter integer WIDTH = 8;

    assign exit = x == 0 | exit_i;
    assign next = x - 1;

endmodule
