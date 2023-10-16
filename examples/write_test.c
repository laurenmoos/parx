#include "stdlib.h"
#include "stdio.h"
#include "stdint.h"
#include "fcntl.h"
#include "unistd.h"
#include "string.h"

const int ARRSIZE = 10;

int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stdin, NULL, _IONBF, 0);
    void* m[ARRSIZE];
    // int fd = open("/dev/urandom", O_RDONLY);
    int fd = open("/home/harden/Documents/harden/examples/testdata", O_RDONLY);
    if (fd < 0) {
        printf("Error opening file\n");
        exit(1);
    }

    for (int i = 0; i < ARRSIZE; i++) {
       m[i] = malloc(0x200);
       memset(m[i], 0x41, 0x200);
    }
    // read(fd, m[1], 0x10);
    // read to local buffer first, then copy over
    // char buf[0x10];
    // if (read(fd, buf, 0x4) != 0x4) {
    //     printf("Error reading from file\n");
    //     exit(1);
    // }
    // printf("%s\n", buf);
    // // memcpy(m[1], buf, 0x10);
    // for (int i = 0; i < 4; i++) {
    //     *(char*)(m[1] + i) = *(buf + i);
    // }

    // // printf("Wrote %lu\n", *(long*)m[1]);
    // printf("Wrote %s\n", (char*)m[1]);
    // *(long*)m[1] = 0x43434343;
    close(fd);
    return 0;
}
