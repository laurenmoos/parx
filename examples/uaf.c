#include "stdio.h"
#include "string.h"
#include "stdlib.h"
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>

FILE* fd_file;

unsigned long tcache_key = 0;

struct LinkedList {
    struct Node* head;
    long count;
};

struct Node {
    struct in_addr ip;
    long idx;
    struct Node* next;
    struct Node* prev;
};

void add_node(struct LinkedList* list, struct Node* node) {
    struct Node* last = list->head;

    if (node == NULL) { return; }

    node->idx = list->count;
    list->count++;
    
    // first node, insert at head
    if (list->count == 1) {
        list->head = node;
        return;
    }

    while (last->next != NULL) {
        last = last->next;
    }
    last->next = node;
    node->prev = last;
}

int new_node(struct LinkedList* list) {
    char ip[0x10];
    struct Node* n = malloc(sizeof(struct Node));
    memset(n, 0, sizeof(struct Node));
    printf("  Enter IP address:\n");
    printf("  > ");
    read(0, ip, 0x10);
    if (!inet_aton(ip, &n->ip)) {
        printf("  [ Error ]: could not parse IP address\n");
        exit(1);
    }
    add_node(list, n);
}

struct Node* find_node(struct LinkedList* list, struct in_addr ip) {
    struct Node* curr = list->head;
    while (curr != NULL) {
        if (curr->ip.s_addr == ip.s_addr) return curr;
        else curr = curr->next;
    }
    return NULL;
}

void delete_node(struct LinkedList* list, struct Node* n) {
    if (n == NULL || list == NULL) {
        return;
    }
    if (list->head == NULL) {
        printf("  [ Error ]: list has no data to delete");
        return;
    }

    if (list->head == n) {
        list->head = n->next;
    } else if (n->prev != NULL) {
        n->prev->next = n->next;
    }
    
    if (n->next != NULL) {
        n->next->prev = n->prev;
    }
    list->count--;
    free(n);
}

int remove_node(struct LinkedList* list) {
    char ip_str[0x10];
    struct in_addr ip;
    printf("  Enter IP address to remove:\n");
    printf("  > ");
    read(0, ip_str, 0x10);
    if (!inet_aton(ip_str, &ip)) {
        printf("  [ Error ]: could not  parse IP address\n");
        exit(1);
    }
    struct Node* n = find_node(list, ip);
    if (n == NULL) {
        printf("  [ Error ]: No matching IP found\n");
        return 1;
    } else {
        delete_node(list, n);
    }
    return 0;
}

void print_list(struct LinkedList* list) {
    struct Node* curr = list->head;
    printf("\nIndex: IP\n");
    while(curr != NULL) {
        printf("  IP index %lu: %s\n", curr->idx, inet_ntoa(curr->ip));
        curr = curr->next;
    };
    printf("\n");
}

int clear_list(struct LinkedList* list) {
    if (list == NULL) { return -1; }
    struct Node* curr = list->head;
    struct Node* next;
    while (curr != NULL) {
        next = curr->next;
        free(curr);
        curr = next;
    }
    list->count = 0;
}

void win() {
    long guess = 0;
    printf("  What is the value of tcache_key?\n");
    printf("  > ");
    scanf("%lu", &guess);
    if (guess == tcache_key) {
        printf("  You win!\n");
        system("/bin/sh");
        exit(0);
    }
    else {
        printf("  Wrong! Please try again\n");
    }
}

long menu() {
    printf("\nPlease enter an option\n");
    printf("  1. Allocate allowlist IP\n");
    printf("  2. Delete allowlist IP\n");
    printf("  3. Print allowlist\n");
    printf("  4. Clear allowlist\n");
    printf("  5. Allocate denylist IP\n");
    printf("  6. Delete denylist IP\n");
    printf("  7. Print denylist\n");
    printf("  8. Clear denylist\n");
    printf("  9. Win\n");
    printf("  10. Quit\n");
    printf("  > ");
    unsigned long choice = 0;
    scanf("%lu", &choice);
    return choice;
}

int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stdin,  NULL, _IONBF, 0);

    // get tcache_key from a freed heap allocation. This a unique psuedo-
    // random number generated after the first free() call and placed at 
    // allocation+0xusize in all freed allocations within tcache
    // leaking this value is a necessary primitive for poisoning tcache in
    // glibc >v2.34

    void* dummy = malloc(0x10);
    free(dummy);
    tcache_key = *(unsigned long*)(dummy + sizeof(long));

    // reallocate chunk so that the heap starts in a "clean" state 
    // with nothing in tcache
    dummy = malloc(0x10);

    fd_file = stdin;
    struct LinkedList allow;
    struct LinkedList deny;
    allow.head = NULL;
    deny.head = NULL;
    allow.count = 0;
    deny.count = 0;
    
    long choice;
    while (1) {
        choice = menu();
        
        switch(choice) {
        case 1: 
            new_node(&allow);
            break;
        case 2:
            remove_node(&allow);
            break;
        case 3:
            print_list(&allow);
            break;
        case 4: 
            clear_list(&allow);
            break;
        case 5: 
            new_node(&deny);
            break;
        case 6:
            remove_node(&deny);
            break;
        case 7:
            print_list(&deny);
            break;
        case 8:
            clear_list(&deny);
            break;
        case 9:
            win();
            break;
        case 10:
            free(dummy);
            return 0;
        default:
            {}
        }
    }
    free(dummy);
    return 0;
}
