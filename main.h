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
#define PASTE_3(a, b, c) a##b##c
#define PASTE_EXPAND_3(a, b, c) PASTE_3(a, b, c)
#define _OR_(a, b) PASTE_EXPAND_3(_OR__, a, b)
#define _AND_(a, b) PASTE_EXPAND_3(_AND__, a, b)
#define _XOR_(a, b) PASTE_EXPAND_3(_XOR__, a, b)
#define SUBMOD(a0, a1, a2, b0, b1, b2)                                         \
  SUBMOD0(b0, a0, _XOR_(b1, a1), _AND_(b0, a0), b2, a2, b1, a1)
#define SUBMOD0(b0, a0, t, t1, b2, a2, b1, a1)                                 \
  SUBMOD1(b0, a0, t, t1, _XOR_(b2, a2), _OR_(_AND_(b1, a1), _AND_(t, t1)), b2, \
          a2)
#define SUBMOD1(b0, a0, t, t1, t0, t2, b2, a2)                                 \
  _XOR_(b0, a0), _XOR_(t, t1), _XOR_(t0, t2), _OR_(_AND_(b2, a2), _AND_(t0, t2))
#define ADDER(a0, a1, a2, a3, b0, b1, b2, b3)                                  \
  ADDER0(b0, a0, _XOR_(b1, a1), _AND_(b0, a0), b2, a2, b1, a1, b3, a3)
#define ADDER0(b0, a0, t, t2, b2, a2, b1, a1, b3, a3)                          \
  ADDER1(b0, a0, t, t2, _XOR_(b2, a2), _OR_(_AND_(b1, a1), _AND_(t, t2)), b3,  \
         a3, b2, a2, b1)
#define ADDER1(b0, a0, t, t2, t0, t4, b3, a3, b2, a2, b1)                      \
  ADDER2(b0, a0, t, t2, _XOR_(t0, t4), b3, a3, b2, a2, t0, t4, b1)
#define ADDER2(b0, a0, t, t2, t5, b3, a3, b2, a2, t0, t4, b1)                  \
  ADDER3(b0, a0, t, t2, t5, b3, a3, b2, a2, t0, t4,                            \
         SUBMOD(b2, a2, t5, b0, b1, b2))
#define ADDER3(b0, a0, t, t2, t5, b3, a3, b2, a2, t0, t4, bundleof4_)          \
  ADDER4(b0, a0, t, t2, t5, _XOR_(b3, a3), _AND_(b2, a2), t0, t4, b3, a3,      \
         bundleof4_)
#define ADDER4(b0, a0, t, t2, t5, t1, t3, t0, t4, b3, a3, t6, t7, t8, t9)      \
  _XOR_(b0, a0), _XOR_(t, t2), t5, _XOR_(t1, _OR_(t3, _AND_(t0, t4))),         \
      _OR_(_OR_(_AND_(b3, a3), _AND_(t1, t3)), _AND_(_AND_(t1, t0), t4)), t6,  \
      t7, t8, t9
