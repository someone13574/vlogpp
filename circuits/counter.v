module counter (
    input clk,
    output reg [3:0] out
);

    wire [3:0] incremented;
    inc inc (
        .in (out),
        .out(incremented)
    );

    always @(posedge clk) begin
        out <= incremented;
    end

endmodule


module inc (
    input  [3:0] in,
    output [3:0] out
);

    assign out = in + 1;

endmodule
