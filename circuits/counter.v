module counter (
    input clk,
    output reg [3:0] out
);

    always @(posedge clk) begin
        out <= out + 1;
    end

endmodule


// module inc (
//     input  [3:0] in,
//     output [3:0] out
// );

//     assign out = in + 1;

// endmodule
