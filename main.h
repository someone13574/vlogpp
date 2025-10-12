#include "macros/gates.h"

/// Inputs: a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], Outputs: out[0],
/// out[1], out[2], out[3], out[4]
#define ADDER(a0, a1, a2, a3, b0, b1, b2, b3)                                  \
  XOR(b0, a0), XOR(XOR(b1, a1), AND(b0, a0)),                                  \
      XOR(XOR(b2, a2), OR(AND(b1, a1), AND(XOR(b1, a1), AND(b0, a0)))),        \
      XOR(XOR(b3, a3),                                                         \
          OR(AND(b2, a2),                                                      \
             AND(XOR(b2, a2),                                                  \
                 OR(AND(b1, a1), AND(XOR(b1, a1), AND(b0, a0)))))),            \
      OR(OR(AND(b3, a3), AND(XOR(b3, a3), AND(b2, a2))),                       \
         AND(AND(XOR(b3, a3), XOR(b2, a2)),                                    \
             OR(AND(b1, a1), AND(XOR(b1, a1), AND(b0, a0)))))

ADDER(1, 0, 0, 0, 1, 0, 0, 0)
