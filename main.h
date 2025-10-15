#define _OR__00 0
#define _OR__01 1
#define _OR__10 1
#define _OR__11 1
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
#define _NOT__0 1
#define _NOT__1 0
#define PASTE_3(a, b, c) a##b##c
#define PASTE_EXPAND_3(a, b, c) PASTE_3(a, b, c)
#define _OR_(a, b) PASTE_EXPAND_3(_OR__, a, b)
#define _AND_(a, b) PASTE_EXPAND_3(_AND__, a, b)
#define _XOR_(a, b) PASTE_EXPAND_3(_XOR__, a, b)
#define PASTE_4(a, b, c, d) a##b##c##d
#define PASTE_EXPAND_4(a, b, c, d) PASTE_4(a, b, c, d)
#define _DFF_P_(c, d, pq) PASTE_EXPAND_4(_DFF_P__, c, d, pq)
#define PASTE_2(a, b) a##b
#define PASTE_EXPAND_2(a, b) PASTE_2(a, b)
#define _NOT_(a) PASTE_EXPAND_2(_NOT__, a)
#define COUNTER(clk, out0i, out2i, out1i, out3i)                               \
  COUNTER0(clk, out0i, out1i, out2i, _AND_(out1i, out0i), out3i)
#define COUNTER0(clk, out0i, out1i, out2i, t, out3i)                           \
  _DFF_P_(clk, _NOT_(out0i), out0i), _DFF_P_(clk, _XOR_(out1i, out0i), out1i), \
      _DFF_P_(clk, _XOR_(out2i, t), out2i),                                    \
      _DFF_P_(clk, _XOR_(out3i, _AND_(out2i, t)), out3i)

COUNTER(1, 0, 1, 0, 0)
