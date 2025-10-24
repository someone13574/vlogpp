`default_nettype none

module counter (
    input var logic clk,
    output var logic [3:0] out,
    output var logic cont
);

    logic [3:0] next;
    inc inc (
        .in (out),
        .out(next)
    );

    always_ff @(posedge clk) begin
        out <= next;
    end

    always_comb begin
        cont = next != 0;
    end

endmodule


module inc (
    input var  logic [3:0] in,
    output var logic [3:0] out
);

    assign out = in + 1;

endmodule
