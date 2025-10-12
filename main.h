#include "macros/gates.h"

/// Cost: 4, Inputs: a, b, c, Outputs: out[0], out[1]
#define _ADDER(c, xor) AND(xor, c), OR(xor, c)
#define ADDER(a, b, c) _ADDER(c, XOR(a, b))

// ADDER(0, 0, 0)
