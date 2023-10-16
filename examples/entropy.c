#include "stdio.h"
#include "stdlib.h"
#include <unistd.h>
#include "stdint.h"
#include <fcntl.h>
#include <string.h>

const unsigned int BUFSIZE = 0x20;
const unsigned int ARRSIZE = 0x10;
int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stdin, NULL, _IONBF, 0);

    void* heap_allocs[ARRSIZE];
    int dev_urand = open("/dev/urandom", O_RDONLY);
    if (dev_urand < 0) {
        printf("[ Error ]: could not open /dev/urandom\n");
        exit(1);
    }
    
    printf("[*] Alloc with random data");
    // fill heap with random data
    for (int i = 0; i < ARRSIZE; i++) {
        heap_allocs[i] = malloc(BUFSIZE - 1);
        if (read(dev_urand, heap_allocs[i], BUFSIZE - 1) != BUFSIZE - 1) {
            printf("[ Error ]: did not read %x bytes into chunk %d\n", BUFSIZE, i);
            exit(1);
        }
    }
   
    // clean up heap, don't free the last alloc so the heap stays unconsolidated
    printf("[*] Free all chunks");
    for (int i = 0; i < ARRSIZE - 1; i++) {
        free(heap_allocs[i]);
    }

    // fill with consistent data, and reuse the tcached 0x10 size chunks
    printf("[*] Alloc with consistent data");
    for (int i = 0; i < ARRSIZE - 1; i++) {
        heap_allocs[i] = malloc(BUFSIZE);
        memset(heap_allocs[i], 0x41, BUFSIZE);
    }

    // clean up heap again
    printf("[*] Final clean");
    for (int i = 0; i < ARRSIZE; i++) {
        free(heap_allocs[i]);
    }
    return 0;
}
