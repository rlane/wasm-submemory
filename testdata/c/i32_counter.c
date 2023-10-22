#include <stdint.h>

static int32_t counter = 0;

int32_t entry() {
    return ++counter;
}
