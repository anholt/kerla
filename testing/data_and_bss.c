// Build with:
// musl-gcc -static -o data_and_bss_static.test data_and_bss.c
// musl-gcc -static-pie -o data_and_bss_static_pie.test data_and_bss.c
// The static-pie version fails, so far.

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

static uint32_t data[2] = {1, 2};
// Uninitialized values should be in the BSS, which is zeroed during ELF loading.
static uint32_t bss[1024];

#define ARRAY_SIZE(x) (sizeof(x) / sizeof(x[0]))
int main(int argc, char **argv)
{
    for (int i = 0; i < ARRAY_SIZE(data); i++)
    {
        if (data[i] != i + 1)
        {
            printf("data[%d] %d != %d", i, data[i], i + 1);
            exit(1);
        }
    }

    for (int i = 0; i < ARRAY_SIZE(bss); i++)
    {
        if (bss[i] != 0)
        {
            exit(1);
        }
    }

    // This line caused a surprising amount of trouble with this test, causing
    // crashing for static-pie and dynamic binaries.
    fprintf(stderr, "output to stderr\n");

    printf("passed\n");

    return 0;
}
