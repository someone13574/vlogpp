#include "macros/gates.h"

/// Cost: 53, Inputs: a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], b[0],
/// b[1], b[2], b[3], b[4], b[5], b[6], b[7], Outputs: out[0], out[1], out[2],
/// out[3], out[4], out[5], out[6], out[7], out[8]
#define ADDER(a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7)  \
  ADDER__0(a0, b0, _XOR(b1, a1), _AND(b0, a0), a1, b1, b2, a2, a3, b3, b4, a4, \
           a5, b5, b6, a6, a7, b7)

#define ADDER__0(a0, b0, _t4, _t0, a1, b1, b2, a2, a3, b3, b4, a4, a5, b5, b6, \
                 a6, a7, b7)                                                   \
  ADDER__1(a0, b0, _t4, _t0, _OR(_AND(b1, a1), _AND(_t4, _t0)), _XOR(b2, a2),  \
           _XOR(b3, a3), _AND(b2, a2), a3, b3, _XOR(b4, a4), _AND(b4, a4),     \
           _XOR(b5, a5), b5, a5, b6, a6, a7, b7)

#define ADDER__1(a0, b0, _t4, _t0, _t6, _t1, _t3, _t5, a3, b3, _t2, _t9, _t10, \
                 b5, a5, b6, a6, a7, b7)                                       \
  ADDER__2(a0, b0, _t4, _t0, _t6, _t1, _t3, _t5,                               \
           _OR(_OR(_AND(b3, a3), _AND(_t3, _t5)), _AND(_AND(_t3, _t1), _t6)),  \
           _t2, _t9, _t10, _AND(_t10, _t2),                                    \
           _OR(_AND(b5, a5), _AND(_t10, _t9)), _XOR(b6, a6), b6, a6,           \
           _XOR(b7, a7), b7, a7)

#define ADDER__2(a0, b0, _t4, _t0, _t6, _t1, _t3, _t5, _t7, _t2, _t9, _t10,    \
                 _t12, _t11, _t8, b6, a6, _t14, b7, a7)                        \
  ADDER__3(a0, b0, _t4, _t0, _t6, _t1, _t3, _t5, _t7, _t2, _t9, _t10,          \
           _OR(_t11, _AND(_t12, _t7)), _t8, _AND(b6, a6), _t14,                \
           _AND(_t14, _t8), _t12, b7, a7, _t11)

#define ADDER__3(a0, b0, _t4, _t0, _t6, _t1, _t3, _t5, _t7, _t2, _t9, _t10,    \
                 _t13, _t8, _t16, _t14, _t15, _t12, b7, a7, _t11)              \
  _XOR(b0, a0), _XOR(_t4, _t0), _XOR(_t1, _t6),                                \
      _XOR(_t3, _OR(_t5, _AND(_t1, _t6))), _XOR(_t2, _t7),                     \
      _XOR(_t10, _OR(_t9, _AND(_t2, _t7))), _XOR(_t8, _t13),                   \
      _XOR(_t14, _OR(_t16, _AND(_t8, _t13))),                                  \
      _OR(_OR(_OR(_AND(b7, a7), _AND(_t14, _t16)), _AND(_t15, _t11)),          \
          _AND(_AND(_t15, _t12), _t7))

ADDER(1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0)
