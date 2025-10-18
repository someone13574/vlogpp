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
