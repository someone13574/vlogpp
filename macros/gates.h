#include "common.h"

#define _AND_00 0
#define _AND_01 0
#define _AND_10 0
#define _AND_11 1
#define _AND(a, b) _EVAL_CAT3(_AND_, a, b)

#define _OR_00 0
#define _OR_01 1
#define _OR_10 1
#define _OR_11 1
#define _OR(a, b) _EVAL_CAT3(_OR_, a, b)

#define _XOR_00 0
#define _XOR_01 1
#define _XOR_10 1
#define _XOR_11 0
#define _XOR(a, b) _EVAL_CAT3(_XOR_, a, b)
