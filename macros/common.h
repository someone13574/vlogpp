// #define EVAL(...) EVAL1(EVAL1(EVAL1(EVAL1(__VA_ARGS__))))
// #define EVAL1(...) EVAL2(EVAL2(EVAL2(EVAL2(__VA_ARGS__))))
// #define EVAL2(...) EVAL3(EVAL3(EVAL3(EVAL3(__VA_ARGS__))))
// #define EVAL3(...) EVAL4(EVAL4(EVAL4(EVAL4(__VA_ARGS__))))
// #define EVAL4(...) EVAL5(EVAL5(EVAL5(EVAL5(__VA_ARGS__))))
// #define EVAL5(...) EVAL6(EVAL6(EVAL6(EVAL6(__VA_ARGS__))))
// #define EVAL6(...) EVAL7(EVAL7(EVAL7(EVAL7(__VA_ARGS__))))
// #define EVAL7(...) __VA_ARGS__

// #define EMPTY()
// #define DEFER(x) x EMPTY()
// #define OBSTRUCT(...) __VA_ARGS__ DEFER(EMPTY)()
