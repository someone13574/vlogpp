#define _NOT__0 1
#define _NOT__1 0
#define _AND__00 0
#define _AND__01 0
#define _AND__10 0
#define _AND__11 1
#define _XOR__00 0
#define _XOR__01 1
#define _XOR__10 1
#define _XOR__11 0
#define _OR__00 0
#define _OR__01 1
#define _OR__10 1
#define _OR__11 1
#define IF_0 EMPTY
#define IF_1 EXPAND
#define PASTE_2(a, b) a##b
#define PASTE_EXPAND_2(a, b) PASTE_2(a, b)
// Module: `$_NOT_`, Inputs: A, Outputs: Y
#define _NOT_(a) PASTE_EXPAND_2(_NOT__, a)
#define PASTE_3(a, b, c) a##b##c
#define PASTE_EXPAND_3(a, b, c) PASTE_3(a, b, c)
// Module: `$_AND_`, Inputs: A, B, Outputs: Y
#define _AND_(a, b) PASTE_EXPAND_3(_AND__, a, b)
// Module: `$_XOR_`, Inputs: A, B, Outputs: Y
#define _XOR_(a, b) PASTE_EXPAND_3(_XOR__, a, b)
// Module: `inc`, Inputs: in[0], in[1], in[2], in[3], Outputs: out[0], out[1], out[2], out[3]
#define INC(in0, in1, in2, in3) INC_0(in0, in1, _AND_(in1, in0), in2, in3)
#define INC_0(in0, in1, t, in2, in3) _NOT_(in0), _XOR_(in1, in0), _XOR_(in2, t), _XOR_(in3, _AND_(in2, t))
// Module: `$_OR_`, Inputs: A, B, Outputs: Y
#define _OR_(a, b) PASTE_EXPAND_3(_OR__, a, b)
// Module: `counter`, Inputs: out[2].i, out[3].i, out[0].i, out[1].i, Outputs: out[0], out[1], out[2], out[3], cont
#define COUNTER(out2i, out3i, out0i, out1i) COUNTER_0(INC(out0i, out1i, out2i, out3i))
#define COUNTER_0(inc) COUNTER_1(inc)
#define COUNTER_1(bt, bt0, bt1, bt2) bt, bt0, bt1, bt2, _OR_(_OR_(bt, bt0), _OR_(bt1, bt2))
#define EMPTY(...)
#define DEFER(x) x EMPTY()
#define OBSTRUCT(...) __VA_ARGS__ DEFER(EMPTY)()
#define EXPAND(...) __VA_ARGS__
#define IF(cont) PASTE_EXPAND_2(IF_, cont)
#define REPEAT_COUNTER_INDIRECT() REPEAT_COUNTER
#define REPEAT_COUNTER(out0i, out1i, out2i, out3i, cont) IF(cont)(__COUNTER__ OBSTRUCT(REPEAT_COUNTER_INDIRECT)()(COUNTER(out2i, out3i, out0i, out1i)))
#define EVAL0(...) __VA_ARGS__
#define EVAL1(...) EVAL0(EVAL0(EVAL0(EVAL0(__VA_ARGS__))))
#define EVAL2(...) EVAL1(EVAL1(EVAL1(EVAL1(__VA_ARGS__))))
#define EVAL3(...) EVAL2(EVAL2(EVAL2(EVAL2(__VA_ARGS__))))
#define EVAL4(...) EVAL3(EVAL3(EVAL3(EVAL3(__VA_ARGS__))))
#define EVAL5(...) EVAL4(EVAL4(EVAL4(EVAL4(__VA_ARGS__))))

EVAL5(REPEAT_COUNTER(0, 0, 0, 0, 1))
