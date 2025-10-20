`default_nettype none

module vlogpp_repeat_dec (
    input var logic [WIDTH - 1:0] x,
    output var logic cont,
    output var logic [WIDTH - 1:0] next
);

    parameter int WIDTH = 8;

    assign cont = x != 1;
    assign next = x - 1;

endmodule
