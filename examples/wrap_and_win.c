#include "stdlib.h"
#include "string.h"
#include "stdio.h"
#include "stdint.h"
#include "unistd.h"
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/wait.h>

const uint8_t GROUP_SIZE = 16;
FILE* fd_file;

long menu() {
    printf("Please enter an option\n");
    printf("  1. Allocate username\n");
    printf("  2. Define group\n");
    printf("  3. Delete username\n");
    printf("  4. Delete group\n");
    printf("  5. Print group\n");
    printf("  6. Win\n");
    printf("  7. Quit\n");
    printf("  > ");
    char c;
    char d = 0;
    c = getc(fd_file);
    while (d != '\n') d = getc(fd_file); // consume '\n'
    return strtol(&c, NULL, 10);
}

char* alloc_uname() {
    unsigned short len;
    printf("    Enter length of username (max 65535)\n");
    printf("     > ");
    fscanf(fd_file, "%hu", &len);
    getc(fd_file); // consume '\n'
    // printf("allocating length %d\n", len + 1);
    char* s = malloc((unsigned short)(len + 1));
    printf("    Enter username\n");
    printf("     > ");
    fgets(s, len, fd_file);
    char* n = strchr(s, '\n');
    if (n > 0) *n = '\0'; // replace '\n' with null terminator
    else {
        while (1) {
            char c = getc(fd_file); // consume '\n'
            if (c == '\n') break;
        }
    }
    // printf("%s\n", s);
    return s;
}

char* alloc_group() {
    char* s = malloc(GROUP_SIZE);
    printf("    Enter group (max %d)\n", GROUP_SIZE);
    printf("     > ");
    fgets(s, GROUP_SIZE, fd_file);
    char* n = strchr(s, '\n');
    if (n > 0) *n = '\0'; // replace '\n' with null terminator
    if (strcmp(s, "admin") == 0) {
        free(s);
        return NULL;
    }
    return s;
}

void free_uname(char* s) {
    free(s);
}

void free_group(char* s) {
    free(s);
}

void win(char* s) {
    if (s != NULL && strcmp(s, "admin") == 0) {
        printf("You won! > ");
        system("/bin/sh");
        exit(0);
    }
    else printf("    Sorry! You didn't win\n");
}

void print_group(char* s) {
    printf("    Name: %s\n", s);
}

int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stdin,  NULL, _IONBF, 0);
    
    unsigned long c;
    char* uname = NULL;
    char* group = NULL;
    
    fd_file = stdin;

    while (1) {
        c = menu();
        switch (c) {
        case 1:
            uname = alloc_uname();
            break;
        case 2:
            group = alloc_group();
            break;
        case 3:
            free_uname(uname);
            uname = NULL;
            break;
        case 4:
            free_group(group);
            group = NULL;
            break;
        case 5:
            print_group(group);
            break;
        case 6:
            win(group);
            break;
        case 7:
            exit(0);
            break;
        default:
            continue;
        }
    }
}
