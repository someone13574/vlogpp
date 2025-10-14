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
#define ADDER(a0, a1, a2, a3, b0, b1, b2, b3)                                  \
  _XOR_(b0, a0), _XOR_(_XOR_(b1, a1), _AND_(b0, a0)),                          \
      _XOR_(_XOR_(b2, a2),                                                     \
            _OR_(_AND_(b1, a1), _AND_(_XOR_(b1, a1), _AND_(b0, a0)))),         \
      _XOR_(_XOR_(b3, a3),                                                     \
            _OR_(_AND_(b2, a2),                                                \
                 _AND_(_XOR_(b2, a2),                                          \
                       _OR_(_AND_(b1, a1),                                     \
                            _AND_(_XOR_(b1, a1), _AND_(b0, a0)))))),           \
      _OR_(_OR_(_AND_(b3, a3), _AND_(_XOR_(b3, a3), _AND_(b2, a2))),           \
           _AND_(_AND_(_XOR_(b3, a3), _XOR_(b2, a2)),                          \
                 _OR_(_AND_(b1, a1), _AND_(_XOR_(b1, a1), _AND_(b0, a0)))))

ADDER(1, 1, 0, 0, 1, 1, 0, 0)
