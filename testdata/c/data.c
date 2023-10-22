#include <stdint.h>

static int32_t data = 42;

int32_t entry() {
    return data;
}

void inc() {
    data++;
}
