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
#define _DFF_P__000 0
#define _DFF_P__001 1
#define _DFF_P__010 0
#define _DFF_P__011 1
#define _DFF_P__100 0
#define _DFF_P__101 0
#define _DFF_P__110 1
#define _DFF_P__111 1
#define _OR__00 0
#define _OR__01 1
#define _OR__10 1
#define _OR__11 1
#define IF_0 EMPTY
#define IF_1 EVAL0
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
// Module: `inc`, Inputs: in[0], in[1], in[2], in[3], Outputs: out[0], out[1],
// out[2], out[3]
#define INC(in0, in1, in2, in3) INC_0(in0, in1, in2, _AND_(in1, in0), in3)
#define INC_0(in0, in1, in2, t, in3)                                           \
  _NOT_(in0), _XOR_(in1, in0), _XOR_(in2, t), _XOR_(in3, _AND_(in2, t))
#define PASTE_4(a, b, c, d) a##b##c##d
#define PASTE_EXPAND_4(a, b, c, d) PASTE_4(a, b, c, d)
// Module: `$_DFF_P_`, Inputs: C, D, pQ, Outputs: Q
#define _DFF_P_(c, d, pq) PASTE_EXPAND_4(_DFF_P__, c, d, pq)
// Module: `counter`, Inputs: clk, out[2].i, out[3].i, out[0].i, out[1].i,
// Outputs: out[0], out[1], out[2], out[3]
#define COUNTER(clk, out0i, out1i, out2i, out3i)                               \
  COUNTER_0(clk, INC(out0i, out1i, out2i, out3i), out0i, out1i, out2i, out3i)
#define COUNTER_0(clk, bundleof4_, out0i, out1i, out2i, out3i)                 \
  COUNTER_1(clk, bundleof4_, out0i, out1i, out2i, out3i)
#define COUNTER_1(clk, bt, bt0, bt1, bt2, out0i, out1i, out2i, out3i)          \
  _DFF_P_(clk, bt, out0i), _DFF_P_(clk, bt0, out1i), _DFF_P_(clk, bt1, out2i), \
      _DFF_P_(clk, bt2, out3i)
// Module: `$_OR_`, Inputs: A, B, Outputs: Y
#define _OR_(a, b) PASTE_EXPAND_3(_OR__, a, b)
// Module: `vlogpp_repeat_dec`, Inputs: x[0], x[1], x[2], x[3], Outputs: cont,
// next[0], next[1], next[2], next[3]
#define VLOGPP_REPEAT_DEC(x0, x1, x2, x3)                                      \
  VLOGPP_REPEAT_DEC_0(x0, x1, x2, x3, _NOT_(x1))
#define VLOGPP_REPEAT_DEC_0(x0, x1, x2, x3, t0)                                \
  VLOGPP_REPEAT_DEC_1(_NOT_(x0), x1, x2, x3, t0, x0, _NOT_(x2),                \
                      _OR_(x1, _AND_(t0, x0)))
#define VLOGPP_REPEAT_DEC_1(t1, x1, x2, x3, t0, x0, t, t2)                     \
  _OR_(_OR_(t1, x1), _OR_(x2, x3)), t1, _XOR_(t0, x0), _XOR_(t, t2),           \
      _XOR_(_NOT_(x3), _OR_(x2, _AND_(t, t2)))
#define EVAL0(...) __VA_ARGS__
#define EVAL1(...) EVAL0(EVAL0(EVAL0(EVAL0(__VA_ARGS__))))
#define EVAL2(...) EVAL1(EVAL1(EVAL1(EVAL1(__VA_ARGS__))))
#define EVAL3(...) EVAL2(EVAL2(EVAL2(EVAL2(__VA_ARGS__))))
#define EVAL4(...) EVAL3(EVAL3(EVAL3(EVAL3(__VA_ARGS__))))
#define EVAL5(...) EVAL4(EVAL4(EVAL4(EVAL4(__VA_ARGS__))))
#define EMPTY(...)
#define DEFER(x) x EMPTY()
#define OBSTRUCT(...) __VA_ARGS__ DEFER(EMPTY)()
#define IF(cont) PASTE_EXPAND_2(IF_, cont)

#define REPEAT(cont, x0, x1, x2, x3, i0, i1, i2, i3)                           \
  IF(cont)                                                                     \
  (<COUNTER(1, i0, i1, i2, i3)> OBSTRUCT(REPEAT_INDIRECT)()(                   \
      VLOGPP_REPEAT_DEC(x0, x1, x2, x3), COUNTER(1, i0, i1, i2, i3)))
#define REPEAT_INDIRECT() REPEAT

EVAL5(REPEAT(1, 1, 0, 1, 0, 0, 0, 0, 0))
