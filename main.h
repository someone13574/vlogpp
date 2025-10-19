#define WHEN_DONE_FINISHED_0 FALSE,
#define WHEN_DONE_FINISHED_1 FALSE,
#define WHEN_DONE_FINISHED_TAIL TRUE,
#define DECREMENT_0
#define DECREMENT_1 0
#define DECREMENT_TAIL TAIL
#define PASTE_2(a, b) a##b
#define PASTE_EXPAND_2(a, b) PASTE_2(a, b)
#define WHEN_DONE_SELECT_TRUE(a, b) a
#define WHEN_DONE_SELECT_FALSE(a, b) b
#define WHEN_DONE_SELECT(select, discard, a, b)                                \
  PASTE_EXPAND_2(WHEN_DONE_SELECT_, select)(a, b)
#define WHEN_DONE_SELECT_EXPAND(bundle, a, b) WHEN_DONE_SELECT(bundle, a, b)
#define WHEN_DONE_FINISHED(count) PASTE_EXPAND_2(WHEN_DONE_FINISHED_, count)
#define WHEN_DONE(count, a, b)                                                 \
  WHEN_DONE_SELECT_EXPAND(WHEN_DONE_FINISHED(count), a, b)
#define DECREMENT(count) PASTE_EXPAND_2(DECREMENT_, count)

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

#define REPEAT(count)                                                          \
  HI WHEN_DONE(count, EAT,                                                     \
               EXPAND)(OBSTRUCT(REPEAT_INDIRECT)()(DECREMENT(count)))
#define REPEAT_INDIRECT() REPEAT

EVAL(REPEAT(1 1 1 1 1 1 TAIL))
