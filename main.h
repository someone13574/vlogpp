#define _NOT__0 1
#define _NOT__1 0
#define _OR__00 0
#define _OR__01 1
#define _OR__10 1
#define _OR__11 1
#define _XOR__00 0
#define _XOR__01 1
#define _XOR__10 1
#define _XOR__11 0
#define _AND__00 0
#define _AND__01 0
#define _AND__10 0
#define _AND__11 1
#define PASTE_2(a, b) a##b
#define PASTE_EXPAND_2(a, b) PASTE_2(a, b)
// Module: `$_NOT_`, Inputs: A, Outputs: Y
#define _NOT_(a) PASTE_EXPAND_2(_NOT__, a)
#define PASTE_3(a, b, c) a##b##c
#define PASTE_EXPAND_3(a, b, c) PASTE_3(a, b, c)
// Module: `$_OR_`, Inputs: A, B, Outputs: Y
#define _OR_(a, b) PASTE_EXPAND_3(_OR__, a, b)
// Module: `$_XOR_`, Inputs: A, B, Outputs: Y
#define _XOR_(a, b) PASTE_EXPAND_3(_XOR__, a, b)
// Module: `$_AND_`, Inputs: A, B, Outputs: Y
#define _AND_(a, b) PASTE_EXPAND_3(_AND__, a, b)
// Module: `vlogpp_repeat_dec`, Inputs: exit_i, x[0], x[1], x[2], x[3], Outputs: exit, next[0], next[1], next[2], next[3]
#define VLOGPP_REPEAT_DEC(exit_i, x0, x1, x2, x3) VLOGPP_REPEAT_DEC_0(x0, x1, x2, x3, exit_i, _NOT_(x1))
#define VLOGPP_REPEAT_DEC_0(x0, x1, x2, x3, exit_i, t0) VLOGPP_REPEAT_DEC_1(x0, x1, x2, x3, exit_i, t0, _NOT_(x2), _OR_(x1, _AND_(t0, x0)))
#define VLOGPP_REPEAT_DEC_1(x0, x1, x2, x3, exit_i, t0, t, t1) _OR_(_NOT_(_OR_(_OR_(x0, x1), _OR_(x2, x3))), exit_i), _NOT_(x0), _XOR_(t0, x0), _XOR_(t, t1), _XOR_(_NOT_(x3), _OR_(x2, _AND_(t, t1)))

#define EMPTY()
#define DEFER(x) x EMPTY()
#define OBSTRUCT(...) __VA_ARGS__ DEFER(EMPTY)()
#define EAT(...)
#define EXPAND(...) __VA_ARGS__

#define EVAL(...) EVAL1(EVAL1(EVAL1(EVAL1(__VA_ARGS__))))
#define EVAL1(...) EVAL2(EVAL2(EVAL2(EVAL2(__VA_ARGS__))))
#define EVAL2(...) EVAL3(EVAL3(EVAL3(EVAL3(__VA_ARGS__))))
#define EVAL3(...) EVAL4(EVAL4(EVAL4(EVAL4(__VA_ARGS__))))
#define EVAL4(...) EVAL5(EVAL5(EVAL5(EVAL5(__VA_ARGS__))))
#define EVAL5(...) __VA_ARGS__

#define WHILE_NOT_0 EXPAND
#define WHILE_NOT_1 EAT
#define WHILE_NOT(exit) PASTE_EXPAND_2(WHILE_NOT_, exit)

#define REPEAT(exit, x0, x1, x2, x3)                                           \
  WHILE_NOT(exit)                                                              \
  (HI OBSTRUCT(REPEAT_INDIRECT)()(VLOGPP_REPEAT_DEC(exit, x0, x1, x2, x3)))
#define REPEAT_INDIRECT() REPEAT

EVAL(REPEAT(0, 1, 1, 0, 0))
