#include "stdlib.h"
#include "stdio.h"
#include "stdint.h"

int main() {
    // setvbuf(stdout, NULL, _IONBF, 0);
    // setvbuf(stdin, NULL, _IONBF, 0);
    long* m = malloc(16);
    *m = 0x41424344;
    // printf("Wrote %lu\n", *m);
    return 0;
}
