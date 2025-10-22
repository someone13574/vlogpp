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
#define PASTE_EXPAND_3(...) PASTE_3(__VA_ARGS__)
// Module: `$_XOR_`, Inputs: A, B, Outputs: Y
#define _XOR_(...) PASTE_EXPAND_3(_XOR__, __VA_ARGS__)
// Module: `$_AND_`, Inputs: A, B, Outputs: Y
#define _AND_(...) PASTE_EXPAND_3(_AND__, __VA_ARGS__)
// Module: `$_OR_`, Inputs: A, B, Outputs: Y
#define _OR_(...) PASTE_EXPAND_3(_OR__, __VA_ARGS__)
// Module: `adder`, Inputs: a[0], a[1], b[0], b[1], b[2], a[2], b[3], a[3], c, b[4], a[4], b[5], a[5], b[6], a[6], b[7], a[7], Outputs: out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7], out[8]
#define ADDER(a0, a1, b0, b1, ...) ADDER_0(b0, a0, _XOR_(b1, a1), _AND_(b0, a0), b1, a1, __VA_ARGS__)
#define ADDER_0(b0, a0, t0, t7, b1, a1, b2, a2, b3, a3, ...) ADDER_1(_XOR_(b0, a0), t0, t7, _XOR_(b2, a2), _OR_(_AND_(b1, a1), _AND_(t0, t7)), _XOR_(b3, a3), _AND_(b2, a2), b3, a3, __VA_ARGS__)
#define ADDER_1(t, t0, t7, t1, t16, t2, t8, b3, a3, c, b4, a4, b5, a5, ...) ADDER_2(_XOR_(t0, t7), _AND_(c, t), t1, t16, t2, t8, _XOR_(b4, a4), _OR_(_OR_(_AND_(b3, a3), _AND_(t2, t8)), _AND_(_AND_(t2, t1), t16)), _XOR_(b5, a5), _AND_(b4, a4), b5, a5, __VA_ARGS__, c, t)
#define ADDER_2(t14, t11, t1, t16, t2, t8, t3, t19, t4, t9, b5, a5, ...) ADDER_3(_XOR_(t1, t16), _AND_(t14, t11), _XOR_(t2, _OR_(t8, _AND_(t1, t16))), _XOR_(t3, t19), _XOR_(t4, _OR_(t9, _AND_(t3, t19))), _OR_(_AND_(b5, a5), _AND_(t4, t9)), _AND_(t4, t3), t19, __VA_ARGS__, t14, t11)
#define ADDER_3(t18, t15, t21, t20, t24, t17, t12, t19, b6, a6, b7, a7, ...) ADDER_4(_AND_(_AND_(t21, t18), t15), _XOR_(b6, a6), _OR_(t17, _AND_(t12, t19)), _AND_(t24, t20), _XOR_(b7, a7), _AND_(b6, a6), __VA_ARGS__, t18, t15, t21, t20, t24, b7, a7, t17, t12, t19)
#define ADDER_4(t25, t5, t22, t26, t6, t10, ...) ADDER_5(t25, _XOR_(t5, t22), _AND_(t26, t25), _XOR_(t6, _OR_(t10, _AND_(t5, t22))), t6, t10, _AND_(t6, t5), t26, __VA_ARGS__)
#define ADDER_5(t25, t23, t28, t27, t6, t10, t13, t26, c, t, t14, t11, t18, t15, t21, t20, t24, b7, a7, t17, t12, t19) _XOR_(c, t), _XOR_(t14, t11), _XOR_(t18, t15), _XOR_(t21, _AND_(t18, t15)), _XOR_(t20, t25), _XOR_(t24, _AND_(t20, t25)), _XOR_(t23, t28), _XOR_(t27, _AND_(t23, t28)), _XOR_(_OR_(_OR_(_OR_(_AND_(b7, a7), _AND_(t6, t10)), _AND_(t13, t17)), _AND_(_AND_(t13, t12), t19)), _AND_(_AND_(_AND_(t27, t23), t26), t25))
