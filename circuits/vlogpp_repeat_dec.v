module vlogpp_repeat_dec (
    input [WIDTH - 1:0] x,
    output cont,
    output [WIDTH - 1:0] next
);

    parameter integer WIDTH = 8;

    assign cont = x != 1;
    assign next = x - 1;

endmodule
