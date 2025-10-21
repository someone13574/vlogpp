`default_nettype none

module adder (
    input var logic [WIDTH - 1:0] a,
    input var logic [WIDTH - 1:0] b,
    input var logic c,
    output var logic [WIDTH:0] out
);

    parameter int WIDTH = 8;

    always_comb begin
        out = a + b + c;
    end

endmodule
