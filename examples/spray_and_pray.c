#include <fcntl.h>
#include <unistd.h>
#include "stdlib.h"
#include "stdio.h"
#include "stdint.h"
#include "string.h"
#include <sys/mman.h>

int urandom = -1;
void* end_heap = 0;
void* start_heap = 0;
unsigned long long points = 0;
void* check_end = 0;

void load_file() {
    // printf("\nPlease enter a choice:\n");
    // printf("  1. Load \"random.txt\"\n");
    // printf("  2. Load \"constant.txt\"\n");
    unsigned long choice = 0;
    int fd = -1;
    void* map;
    scanf("%lu", &choice);
    switch(choice) {
    case 1:
        map = mmap(0, 0x1000, PROT_READ | PROT_WRITE, MAP_ANON | MAP_PRIVATE, -1, 0);
        fd = open("/dev/urandom", O_RDONLY);
        if (read(fd, map, 0x1000) != 0x1000) {
            printf("[ Error ]: could not read 0x1000 bytes from /dev/urandom");
            exit(1);
        }
        break;
    case 2:
        fd = open("constant.txt", O_RDONLY);
        map = mmap(0, 0x1000, PROT_READ | PROT_WRITE, MAP_PRIVATE, fd, 0);
        break;
    default:
        printf("  [ Error ]: invalid choice\n");
        return;
    }
    if (fd == -1) {
        printf("[ Error ]: could not open file\n");
        exit(1);
    }
    if (map == MAP_FAILED) {
        printf("[ Error ]: failed mmap\n");
        exit(1);
    }
    if (fd != -1) close(fd);
}

void big_chunk_big_entropy() {
    void* a = malloc(0x100);
    ssize_t b = read(urandom, a, 0x100);
    if (b != 0x100) {
        printf("Error reading in big_chunk_big_entropy()\n");
        exit(1);
    }
    unsigned long long size = *(unsigned long long*)(a-sizeof(long long));
    end_heap = a + (size & ~1) - sizeof(long long);
}

void small_chunk_big_entropy() {
    void* a = malloc(0x20);
    ssize_t b = read(urandom, a, 0x20);
    if (b != 0x20) {
        printf("Error reading in small_chunk_big_entropy()\n");
        exit(1);
    }
    unsigned long long size = *(unsigned long long*)(a-sizeof(long long));
    end_heap = a + (size & ~1) - sizeof(long long);
}

void* big_chunk_medium_entropy() {
    void* a = malloc(0x100);
    ssize_t b = read(urandom, a, 0x80);
    if (b != 0x80) {
        printf("Error reading in big_chunk_medium_entropy()\n");
        exit(1);
    }

    memset(a+0x80, 0x41, 0x80);
    unsigned long long size = *(unsigned long long*)(a-sizeof(long long));
    end_heap = a + (size & ~1) - sizeof(long long);
    return a;
}

void* small_chunk_medium_entropy() {
    void* a = malloc(0x20);
    ssize_t b = read(urandom, a, 0x10);
    if (b != 0x10) {
        printf("Error reading in small_chunk_medium_entropy()\n");
        exit(1);
    }

    memset(a+0x10, 0x41, 0x10);
    unsigned long long size = *(unsigned long long*)(a-sizeof(long long));
    end_heap = a + (size & ~1) - sizeof(long long);

    return a;
}

void* big_chunk_no_entropy() {
    void* a = malloc(0x100);
    memset(a, 0x0, 0x100);
    unsigned long long size = *(unsigned long long*)(a-sizeof(long long));
    end_heap = a + (size & ~1) - sizeof(long long);

    return a;
}

void* small_chunk_no_entropy() {
    void* a = malloc(0x20);
    memset(a, 0x0, 0x20);
    unsigned long long size = *(unsigned long long*)(a-sizeof(long long));
    end_heap = a + (size & ~1) - sizeof(long long);

    return a;
}

void update_points() {
    while ((check_end + 0x100) < end_heap) {
        if (*(unsigned long long*)(check_end + 0x100) == 
            (unsigned long long)0x4141414141414141) {
            points += 1;
        }
        check_end += 0x100;
    }

    printf("You have %llu points\n", points);
}

unsigned long menu() {
    // printf("\nPlease enter an option\n");
    // printf("  1. Allocate some bytes\n");
    // printf("  2. Allocate some bytes\n");
    // printf("  3. Allocate some bytes\n");
    // printf("  4. Allocate some bytes\n");
    // printf("  5. Allocate some bytes\n");
    // printf("  6. Allocate some bytes\n");
    // printf("  7. Check points\n");
    // printf("  8. Quit\n");
    // printf("  > ");
    unsigned long choice = 0;
    scanf("%lu", &choice);
    return choice;
}

int main() {
    uint32_t x = 1024;
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stdin, NULL, _IONBF, 0);

    // start_heap = sbrk(0);
    // end_heap = start_heap;
    // check_end = start_heap;
    // printf("start of heap %p\n", start_heap);
    urandom = open("/dev/urandom", O_RDONLY);

    if (urandom == -1) {
        printf("Error opening file\n");
        exit(1);
    }

    unsigned long choice;
    while (1) {
        // printf("end of heap: %p\n", end_heap);
        choice = menu();
        switch(choice) {
        case 1:;
            long* p;
            p =small_chunk_medium_entropy();
            x--;
            break;
        case 2:
            big_chunk_no_entropy();
            break;
        case 3:
            small_chunk_big_entropy();
            break;
        case 4:
            big_chunk_medium_entropy();
            break;
        case 5:
            big_chunk_big_entropy();
            break;
        case 6:
            small_chunk_no_entropy();
            break;
        case 7:
            update_points();
            break;
        case 8:
            load_file();
            break;
        case 9:
            return 0;
        default:
            {}
        }
    }
    return 0;
}
