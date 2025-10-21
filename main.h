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
// Module: `adder`, Inputs: a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], c, Outputs: out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7], out[8]
#define ADDER(a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7, c) ADDER_0(c, b0, a0, _XOR_(b1, a1), _AND_(b0, a0), b2, a2, b1, a1, b3, a3, b4, a4, b5, a5, b6, a6, b7, a7)
#define ADDER_0(c, b0, a0, t0, t7, b2, a2, b1, a1, b3, a3, b4, a4, b5, a5, b6, a6, b7, a7) ADDER_1(c, _XOR_(b0, a0), t0, t7, _XOR_(b2, a2), _OR_(_AND_(b1, a1), _AND_(t0, t7)), _XOR_(b3, a3), _AND_(b2, a2), b4, a4, b3, a3, b5, a5, b6, a6, b7, a7)
#define ADDER_1(c, t, t0, t7, t1, t16, t2, t8, b4, a4, b3, a3, b5, a5, b6, a6, b7, a7) ADDER_2(c, t, _XOR_(t0, t7), _AND_(c, t), t1, t16, t2, t8, _XOR_(b4, a4), _OR_(_OR_(_AND_(b3, a3), _AND_(t2, t8)), _AND_(_AND_(t2, t1), t16)), _XOR_(b5, a5), _AND_(b4, a4), b6, a6, b5, a5, b7, a7)
#define ADDER_2(c, t, t14, t11, t1, t16, t2, t8, t3, t19, t4, t9, b6, a6, b5, a5, b7, a7) ADDER_3(c, t, t14, t11, _XOR_(t1, t16), _AND_(t14, t11), _XOR_(t2, _OR_(t8, _AND_(t1, t16))), _XOR_(t3, t19), _XOR_(t4, _OR_(t9, _AND_(t3, t19))), b6, a6, _OR_(_AND_(b5, a5), _AND_(t4, t9)), _AND_(t4, t3), t19, b7, a7)
#define ADDER_3(c, t, t14, t11, t18, t15, t21, t20, t24, b6, a6, t17, t12, t19, b7, a7) ADDER_4(c, t, t14, t11, t18, t15, t21, t20, _AND_(_AND_(t21, t18), t15), t24, _XOR_(b6, a6), _OR_(t17, _AND_(t12, t19)), _AND_(t24, t20), _XOR_(b7, a7), _AND_(b6, a6), b7, a7, t17, t12, t19)
#define ADDER_4(c, t, t14, t11, t18, t15, t21, t20, t25, t24, t5, t22, t26, t6, t10, b7, a7, t17, t12, t19) ADDER_5(c, t, t14, t11, t18, t15, t21, t20, t25, t24, _XOR_(t5, t22), _AND_(t26, t25), _XOR_(t6, _OR_(t10, _AND_(t5, t22))), b7, a7, t6, t10, _AND_(t6, t5), t17, t12, t19, t26)
#define ADDER_5(c, t, t14, t11, t18, t15, t21, t20, t25, t24, t23, t28, t27, b7, a7, t6, t10, t13, t17, t12, t19, t26) _XOR_(c, t), _XOR_(t14, t11), _XOR_(t18, t15), _XOR_(t21, _AND_(t18, t15)), _XOR_(t20, t25), _XOR_(t24, _AND_(t20, t25)), _XOR_(t23, t28), _XOR_(t27, _AND_(t23, t28)), _XOR_(_OR_(_OR_(_OR_(_AND_(b7, a7), _AND_(t6, t10)), _AND_(t13, t17)), _AND_(_AND_(t13, t12), t19)), _AND_(_AND_(_AND_(t27, t23), t26), t25))
