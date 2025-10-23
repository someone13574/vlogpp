`default_nettype none

module submod_state (
    input var logic clk,
    output var logic [7:0] cnt
);

    logic [3:0] sub_out;
    stateful_sub sub (
        .clk(clk),
        .sub_cnt(sub_out)
    );

    always_ff @(posedge clk) begin
        cnt <= cnt + sub_out;
    end

endmodule

module stateful_sub (
    input var logic clk,
    output var logic [3:0] sub_cnt
);

    always_ff @(posedge clk) begin
        sub_cnt <= sub_cnt + 1;
    end

endmodule
