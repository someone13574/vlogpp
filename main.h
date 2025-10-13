#include "macros/gates.h"

/// Cost: 33, Inputs: a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], c,
/// Outputs: out[0], out[1], out[2], out[3], out[4]
#define ADDER(a0, a1, a2, a3, b0, b1, b2, b3, c)                               \
  _ADDER_SPLIT_1(c, XOR(b0, a0), XOR(b1, a1), AND(b0, a0), b1, a1,             \
                 XOR(b2, a2), AND(b2, a2), XOR(b3, a3), a3, b3)

#define _ADDER_SPLIT_1(c, temp_var_16, temp_var_17, temp_var_20, b1, a1,       \
                       temp_var_18, temp_var_22, temp_var_19, a3, b3)          \
  _ADDER_SPLIT_2(c, temp_var_16, AND(c, temp_var_16),                          \
                 XOR(temp_var_17, temp_var_20),                                \
                 OR(AND(b1, a1), AND(temp_var_17, temp_var_20)), temp_var_18,  \
                 temp_var_22, temp_var_19, a3, b3)

#define _ADDER_SPLIT_2(c, temp_var_16, temp_var_25, temp_var_24, temp_var_35,  \
                       temp_var_18, temp_var_22, temp_var_19, a3, b3)          \
  _ADDER_SPLIT_3(                                                              \
      c, temp_var_16, temp_var_25, temp_var_24, XOR(temp_var_18, temp_var_35), \
      AND(temp_var_24, temp_var_25),                                           \
      XOR(temp_var_19, OR(temp_var_22, AND(temp_var_18, temp_var_35))),        \
      temp_var_18, a3, temp_var_35, temp_var_22, b3, temp_var_19)

#define _ADDER_SPLIT_3(c, temp_var_16, temp_var_25, temp_var_24, temp_var_26,  \
                       temp_var_27, temp_var_28, temp_var_18, a3, temp_var_35, \
                       temp_var_22, b3, temp_var_19)                           \
  XOR(c, temp_var_16), XOR(temp_var_24, temp_var_25),                          \
      XOR(temp_var_26, temp_var_27),                                           \
      XOR(temp_var_28, AND(temp_var_26, temp_var_27)),                         \
      XOR(OR(OR(AND(b3, a3), AND(temp_var_19, temp_var_22)),                   \
             AND(AND(temp_var_19, temp_var_18), temp_var_35)),                 \
          AND(AND(temp_var_28, temp_var_26), temp_var_27))
