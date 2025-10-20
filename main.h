#define _XOR__00 0
#define _XOR__01 1
#define _XOR__10 1
#define _XOR__11 0
#define _AND__00 0
#define _AND__01 0
#define _AND__10 0
#define _AND__11 1
#define _OR__00 0
#define _OR__01 1
#define _OR__10 1
#define _OR__11 1
#define PASTE_3(a, b, c) a##b##c
#define PASTE_EXPAND_3(a, b, c) PASTE_3(a, b, c)
// Module: `$_XOR_`, Inputs: A, B, Outputs: Y
#define _XOR_(a, b) PASTE_EXPAND_3(_XOR__, a, b)
// Module: `$_AND_`, Inputs: A, B, Outputs: Y
#define _AND_(a, b) PASTE_EXPAND_3(_AND__, a, b)
// Module: `$_OR_`, Inputs: A, B, Outputs: Y
#define _OR_(a, b) PASTE_EXPAND_3(_OR__, a, b)
// Module: `adder`, Inputs: a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7],
// a[8], a[9], a[10], a[11], a[12], a[13], a[14], a[15], b[0], b[1], b[2], b[3],
// b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15],
// c, Outputs: out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7],
// out[8], out[9], out[10], out[11], out[12], out[13], out[14], out[15], out[16]
#define ADDER(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, \
              a15, b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, \
              b14, b15, c)                                                     \
  ADDER_0(c, b0, a0, _XOR_(b1, a1), _AND_(b0, a0), b2, a2, b1, a1, b3, a3, b4, \
          a4, b5, a5, b6, a6, b7, a7, b8, a8, b9, a9, b10, a10, b11, a11, b12, \
          a12, b13, a13, b14, a14, b15, a15)
#define ADDER_0(c, b0, a0, t0, t15, b2, a2, b1, a1, b3, a3, b4, a4, b5, a5,    \
                b6, a6, b7, a7, b8, a8, b9, a9, b10, a10, b11, a11, b12, a12,  \
                b13, a13, b14, a14, b15, a15)                                  \
  ADDER_1(c, b0, a0, t0, t15, _XOR_(b2, a2),                                   \
          _OR_(_AND_(b1, a1), _AND_(t0, t15)), _XOR_(b3, a3), _AND_(b2, a2),   \
          _XOR_(b4, a4), b3, a3, _XOR_(b5, a5), _AND_(b4, a4), _XOR_(b6, a6),  \
          b5, a5, _XOR_(b7, a7), b6, a6, b8, a8, b7, a7, b9, a9, b10, a10,     \
          b11, a11, b12, a12, b13, a13, b14, a14, b15, a15)
#define ADDER_1(c, b0, a0, t0, t15, t1, t34, t2, t16, t3, b3, a3, t4, t17, t5, \
                b5, a5, t6, b6, a6, b8, a8, b7, a7, b9, a9, b10, a10, b11,     \
                a11, b12, a12, b13, a13, b14, a14, b15, a15)                   \
  ADDER_2(                                                                     \
      c, _XOR_(b0, a0), t0, t15, t1, t34, t2, t16, t3,                         \
      _OR_(_OR_(_AND_(b3, a3), _AND_(t2, t16)), _AND_(_AND_(t2, t1), t34)),    \
      t4, t17, t5, _OR_(_AND_(b5, a5), _AND_(t4, t17)), _AND_(t4, t3), t6,     \
      _AND_(b6, a6), _XOR_(b8, a8), b7, a7, _AND_(t6, t5), _XOR_(b9, a9),      \
      _AND_(b8, a8), _XOR_(b10, a10), b9, a9, _XOR_(b11, a11), b10, a10, b12,  \
      a12, b11, a11, b13, a13, b14, a14, b15, a15)
#define ADDER_2(c, t, t0, t15, t1, t34, t2, t16, t3, t39, t4, t17, t5, t35,    \
                t24, t6, t18, t7, b7, a7, t25, t8, t19, t9, b9, a9, t10, b10,  \
                a10, b12, a12, b11, a11, b13, a13, b14, a14, b15, a15)         \
  ADDER_3(c, t, _XOR_(t0, t15), _AND_(c, t), t1, t34, t2, t16, t3, t39, t4,    \
          t17, t5, t35, t24, t6, t18, t7,                                      \
          _OR_(_OR_(_OR_(_AND_(b7, a7), _AND_(t6, t18)), _AND_(t25, t35)),     \
               _AND_(_AND_(t25, t24), t39)),                                   \
          t8, t19, t9, _OR_(_AND_(b9, a9), _AND_(t8, t19)), _AND_(t8, t7),     \
          t10, _AND_(b10, a10), b12, a12, b11, a11, _AND_(t10, t9), b13, a13,  \
          b14, a14, b15, a15)
#define ADDER_3(c, t, t30, t23, t1, t34, t2, t16, t3, t39, t4, t17, t5, t35,   \
                t24, t6, t18, t7, t43, t8, t19, t9, t36, t26, t10, t20, b12,   \
                a12, b11, a11, t27, b13, a13, b14, a14, b15, a15)              \
  ADDER_4(c, t, t30, t23, _XOR_(t1, t34), _AND_(t30, t23),                     \
          _XOR_(t2, _OR_(t16, _AND_(t1, t34))), _XOR_(t3, t39),                \
          _XOR_(t4, _OR_(t17, _AND_(t3, t39))), t5,                            \
          _OR_(t35, _AND_(t24, t39)), t6, t18, _XOR_(t7, t43),                 \
          _XOR_(t8, _OR_(t19, _AND_(t7, t43))), t9,                            \
          _OR_(t36, _AND_(t26, t43)), t10, t20, b12, a12,                      \
          _OR_(_OR_(_AND_(b11, a11), _AND_(t10, t20)), _AND_(t27, t36)),       \
          _AND_(t27, t26), t43, b13, a13, b14, a14, b15, a15)
#define ADDER_4(c, t, t30, t23, t38, t33, t42, t41, t47, t5, t44, t6, t18,     \
                t45, t54, t9, t50, t10, t20, b12, a12, t40, t31, t43, b13,     \
                a13, b14, a14, b15, a15)                                       \
  ADDER_5(c, t, t30, t23, t38, t33, t42, t41, _AND_(_AND_(t42, t38), t33),     \
          t47, _XOR_(t5, t44), _AND_(t47, t41),                                \
          _XOR_(t6, _OR_(t18, _AND_(t5, t44))), t45, t54, _XOR_(t9, t50),      \
          _AND_(t54, t45), _XOR_(t10, _OR_(t20, _AND_(t9, t50))),              \
          _XOR_(b12, a12), _OR_(t40, _AND_(t31, t43)), _XOR_(b13, a13),        \
          _AND_(b12, a12), b14, a14, b13, a13, b15, a15, t40, t31, t43)
#define ADDER_5(c, t, t30, t23, t38, t33, t42, t41, t48, t47, t46, t51, t55,   \
                t45, t54, t53, t58, t61, t11, t49, t12, t21, b14, a14, b13,    \
                a13, b15, a15, t40, t31, t43)                                  \
  ADDER_6(c, t, t30, t23, t38, t33, t42, t41, t48, t47, t46, t51, t55, t45,    \
          _AND_(_AND_(_AND_(t55, t46), t51), t48), t54, t53, t58, t61,         \
          _XOR_(t11, t49), _AND_(_AND_(t61, t53), t58),                        \
          _XOR_(t12, _OR_(t21, _AND_(t11, t49))), _XOR_(b14, a14),             \
          _OR_(_AND_(b13, a13), _AND_(t12, t21)), _AND_(t12, t11), t49,        \
          _XOR_(b15, a15), b14, a14, b15, a15, t40, t31, t43)
#define ADDER_6(c, t, t30, t23, t38, t33, t42, t41, t48, t47, t46, t51, t55,   \
                t45, t63, t54, t53, t58, t61, t52, t65, t60, t13, t37, t28,    \
                t49, t14, b14, a14, b15, a15, t40, t31, t43)                   \
  ADDER_7(c, t, t30, t23, t38, t33, t42, t41, t48, t47, t46, t51, t55, t45,    \
          t63, t54, t53, t58, t61, t52, _AND_(t65, t63), t60, t13,             \
          _OR_(t37, _AND_(t28, t49)), _AND_(t60, t52), t14, _AND_(b14, a14),   \
          b15, a15, _AND_(t14, t13), t37, t28, t40, t31, t43, t65)
#define ADDER_7(c, t, t30, t23, t38, t33, t42, t41, t48, t47, t46, t51, t55,   \
                t45, t63, t54, t53, t58, t61, t52, t67, t60, t13, t57, t62,    \
                t14, t22, b15, a15, t29, t37, t28, t40, t31, t43, t65)         \
  ADDER_8(c, t, t30, t23, t38, t33, t42, t41, t48, t47, t46, _AND_(t51, t48),  \
          t55, t45, t63, t54, t53, _AND_(t58, t63), t61, t52, t67, t60,        \
          _XOR_(t13, t57), _AND_(t62, t67),                                    \
          _XOR_(t14, _OR_(t22, _AND_(t13, t57))), b15, a15, t14, t22, t29,     \
          t37, _AND_(t29, t28), t40, t31, t43, t62, t65)
#define ADDER_8(c, t, t30, t23, t38, t33, t42, t41, t48, t47, t46, t56, t55,   \
                t45, t63, t54, t53, t66, t61, t52, t67, t60, t59, t68, t64,    \
                b15, a15, t14, t22, t29, t37, t32, t40, t31, t43, t62, t65)    \
  _XOR_(c, t), _XOR_(t30, t23), _XOR_(t38, t33), _XOR_(t42, _AND_(t38, t33)),  \
      _XOR_(t41, t48), _XOR_(t47, _AND_(t41, t48)), _XOR_(t46, t56),           \
      _XOR_(t55, _AND_(t46, t56)), _XOR_(t45, t63),                            \
      _XOR_(t54, _AND_(t45, t63)), _XOR_(t53, t66),                            \
      _XOR_(t61, _AND_(t53, t66)), _XOR_(t52, t67),                            \
      _XOR_(t60, _AND_(t52, t67)), _XOR_(t59, t68),                            \
      _XOR_(t64, _AND_(t59, t68)),                                             \
      _XOR_(_OR_(_OR_(_OR_(_OR_(_AND_(b15, a15), _AND_(t14, t22)),             \
                           _AND_(t29, t37)),                                   \
                      _AND_(t32, t40)),                                        \
                 _AND_(_AND_(t32, t31), t43)),                                 \
            _AND_(_AND_(_AND_(_AND_(t64, t59), t62), t65), t63))
