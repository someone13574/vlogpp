// module adder (
//     input  [3:0] a,
//     input  [3:0] b,
//     output [4:0] c,
//     output [4:0] d
// );

//     assign c = a + b;

//     wire [3:0] a_in;
//     assign a_in = {c[2], a[2], b[2], c[0]};

//     submod submod (
//         .a  (a_in),
//         .b  (b),
//         .out(d)
//     );

// endmodule


module adder (
    input  [3:0] a,
    input  [3:0] b,
    output [4:0] out
);
    assign out = a + b;
endmodule
