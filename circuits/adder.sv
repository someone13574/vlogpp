`default_nettype none

module adder #(
    parameter int WIDTH = 4
) (
    input var logic [WIDTH - 1:0] a,
    input var logic [WIDTH - 1:0] b,
    input var logic c,
    output var logic [WIDTH:0] out
);

    always_comb begin
        out = a + b + c;
    end

endmodule
