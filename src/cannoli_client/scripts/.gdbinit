# add `set auto-load safe-path /` to ~./gdbinit
set pagination off
set print address off
set $_exitcode = -999

define print_sbrk
    bt
    printf "sbrk 0x%lx", $rdi
    set $size = $rdi
    printf "\n"
    finish
    printf "sbrk returned\n"
    printf "new heap end: 0x%lx\n", $size + $rax
    c
end

define print_mmap
    printf "mmap "
    i r
    printf "\n"
    c
end

define print_brk
    printf "brk 0x%lx", $rdi
    printf "\n"
    finish
    printf "brk returned 0x%lx\n", $rax
    c
end

b sbrk
commands
    print_sbrk
end

# b brk
# commands
#     print_brk
# end

b mmap
commands
    print_mmap
end

r

if $_exitcode != -999
  quit
end
