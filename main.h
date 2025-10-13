#include "macros/gates.h"

/// Cost: 22, Inputs: a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], Outputs:
/// out[0], out[1], out[2], out[3], out[4]
#define ADDER(a0, a1, a2, a3, b0, b1, b2, b3)                                  \
  _ADDER_SPLIT_1(b0, a0, AND(b0, a0), XOR(b1, a1), b1, a1, XOR(b2, a2),        \
                 AND(b2, a2), XOR(b3, a3), a3, b3)

#define _ADDER_SPLIT_1(b0, a0, temp_var_18, temp_var_17, b1, a1, temp_var_25,  \
                       temp_var_15, temp_var_20, a3, b3)                       \
  _ADDER_SPLIT_2(b0, a0, temp_var_18, temp_var_17,                             \
                 OR(AND(b1, a1), AND(temp_var_17, temp_var_18)), temp_var_25,  \
                 temp_var_15, temp_var_20, a3, b3)

#define _ADDER_SPLIT_2(b0, a0, temp_var_18, temp_var_17, temp_var_23,          \
                       temp_var_25, temp_var_15, temp_var_20, a3, b3)          \
  XOR(b0, a0), XOR(temp_var_17, temp_var_18), XOR(temp_var_25, temp_var_23),   \
      XOR(temp_var_20, OR(temp_var_15, AND(temp_var_25, temp_var_23))),        \
      OR(OR(AND(b3, a3), AND(temp_var_20, temp_var_15)),                       \
         AND(AND(temp_var_20, temp_var_25), temp_var_23))
