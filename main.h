#define OR_00 0
#define OR_01 1
#define OR_10 1
#define OR_11 1
#define AND_00 0
#define AND_01 1
#define AND_10 1
#define AND_11 0
#define XOR_00 0
#define XOR_01 1
#define XOR_10 1
#define XOR_11 0
#define PASTE_3(a, b, c) a##b##c
#define PASTE_EXPAND_3(a, b, c) PASTE_3(a, b, c)
#define OR(a, b) PASTE_EXPAND_3(OR_, a, b)
#define AND(a, b) PASTE_EXPAND_3(AND_, a, b)
#define XOR(a, b) PASTE_EXPAND_3(XOR_, a, b)
