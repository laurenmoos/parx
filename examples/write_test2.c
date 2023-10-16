#include "stdlib.h"
#include "stdio.h"
#include "stdint.h"
#include "fcntl.h"
#include "unistd.h"
#include "string.h"

void alloc(void** m) {
}

int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stdin, NULL, _IONBF, 0);
    void* m[10];
    int fd = open("/home/harden/Documents/harden/examples/testdata", O_RDONLY);
    if (fd < 0) {
        printf("Error opening file\n");
        exit(1);
    }

    for (int i = 0; i < 10; i++) {
        m[i] = malloc(0x10);
        *(long*)m[i] = 0x41414141;
    }

    // test read to heap buffer
    if (read(fd, m[1], 0x4) != 0x4) {
        printf("Error reading from file\n");
        exit(1);
    }
    printf("Wrote %s\n", (char*)m[1]);
    *(long*)m[1] = 0x43434343;

    // test read to stack buffer
    char buf[0x10];
    if (read(fd, buf, 4) != 0x4) {
        printf("Error reading from file\n");
        exit(1);
    }

    // print value to confirm JITted address matches the syscall addr
    printf("%s\n", buf);
    close(fd);
    return 0;
}
