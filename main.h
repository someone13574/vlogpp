#include "macros/gates.h"

/// Cost: 53, Inputs: a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], b[0],
/// b[1], b[2], b[3], b[4], b[5], b[6], b[7], Outputs: out[0], out[1], out[2],
/// out[3], out[4], out[5], out[6], out[7], out[8]
#define ADDER(a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7)  \
  _ADDER_SPLIT_1(a0, b0, AND(b0, a0), XOR(b1, a1), a1, b1, b2, a2, b3, a3, a4, \
                 b4, b5, a5, b6, a6, a7, b7)

#define _ADDER_SPLIT_1(a0, b0, tmp32, tmp40, a1, b1, b2, a2, b3, a3, a4, b4,   \
                       b5, a5, b6, a6, a7, b7)                                 \
  _ADDER_SPLIT_2(a0, b0, tmp32, tmp40, OR(AND(b1, a1), AND(tmp40, tmp32)),     \
                 XOR(b2, a2), AND(b2, a2), XOR(b3, a3), XOR(b4, a4), a3, b3,   \
                 XOR(b5, a5), AND(b4, a4), b6, a6, b5, a5, a7, b7)

#define _ADDER_SPLIT_2(a0, b0, tmp32, tmp40, tmp46, tmp54, tmp34, tmp27,       \
                       tmp28, a3, b3, tmp29, tmp36, b6, a6, b5, a5, a7, b7)    \
  _ADDER_SPLIT_3(                                                              \
      a0, b0, tmp32, tmp40, tmp46, tmp54, tmp34, tmp27, tmp28,                 \
      OR(OR(AND(b3, a3), AND(tmp27, tmp34)), AND(AND(tmp27, tmp54), tmp46)),   \
      tmp29, tmp36, XOR(b6, a6), AND(tmp29, tmp28),                            \
      OR(AND(b5, a5), AND(tmp29, tmp36)), XOR(b7, a7), a6, b6, b7, a7)

#define _ADDER_SPLIT_3(a0, b0, tmp32, tmp40, tmp46, tmp54, tmp34, tmp27,       \
                       tmp28, tmp52, tmp29, tmp36, tmp30, tmp55, tmp49, tmp31, \
                       a6, b6, b7, a7)                                         \
  _ADDER_SPLIT_4(a0, b0, tmp32, tmp40, tmp46, tmp54, tmp34, tmp27, tmp28,      \
                 tmp52, tmp29, tmp36, tmp30, OR(tmp49, AND(tmp55, tmp52)),     \
                 tmp31, AND(b6, a6), tmp49, AND(tmp31, tmp30), b7, a7, tmp55)

#define _ADDER_SPLIT_4(a0, b0, tmp32, tmp40, tmp46, tmp54, tmp34, tmp27,       \
                       tmp28, tmp52, tmp29, tmp36, tmp30, tmp59, tmp31, tmp38, \
                       tmp49, tmp48, b7, a7, tmp55)                            \
  XOR(b0, a0), XOR(tmp40, tmp32), XOR(tmp54, tmp46),                           \
      XOR(tmp27, OR(tmp34, AND(tmp54, tmp46))), XOR(tmp28, tmp52),             \
      XOR(tmp29, OR(tmp36, AND(tmp28, tmp52))), XOR(tmp30, tmp59),             \
      XOR(tmp31, OR(tmp38, AND(tmp30, tmp59))),                                \
      OR(OR(OR(AND(b7, a7), AND(tmp31, tmp38)), AND(tmp48, tmp49)),            \
         AND(AND(tmp48, tmp55), tmp52))

ADDER(1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0)
