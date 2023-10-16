# sudo apt-get install -y gcc-mips-linux-gnu
# sudo apt-get install -y gcc-mipsel-linux-gnu

CFLAGS = -O0 -g -static -fno-pic

all: clean uaf wrap_and_win spray_and_pray writes entropy flip12 target

clean_uaf:
	rm -f ./uaf
	rm -f ./uaf_mips

clean_spray_and_pray:
	rm -f ./spray_and_pray
	rm -f ./spray_and_pray_mips_le

clean_writes:
	rm -f ./write_long
	rm -f ./write_test
	rm -f ./write_test2
	rm -f ./write_long_mips_le
	rm -f ./write_test_mips_le
	rm -f ./write_test2_mips_le
	rm -f ./write_long_mips_be
	rm -f ./write_test_mips_be
	rm -f ./write_test2_mips_be

clean_entropy:
	rm -f ./entropy_mips_le
	rm -f ./entropy

clean_wrap_and_win:
	rm -f ./wrap_and_win_mips_le
	rm -f ./wrap_and_win

clean_flip12:
	@/bin/rm -f ./flip12x

clean_target:
	@/bin/rm -f ./targetx

uaf: clean_uaf
	gcc -o ./uaf $(CFLAGS) ./uaf.c
	mipsel-linux-gnu-gcc -o ./uaf_mips $(CFLAGS) ./uaf.c

spray_and_pray: clean_spray_and_pray
	gcc -o ./spray_and_pray $(CFLAGS) ./spray_and_pray.c
	mipsel-linux-gnu-gcc -o ./spray_and_pray_mips_le $(CFLAGS) ./spray_and_pray.c

writes: clean_writes
	gcc -o ./write_long $(CFLAGS) ./write_long.c
	gcc -o ./write_test $(CFLAGS) ./write_test.c
	gcc -o ./write_test2 $(CFLAGS) ./write_test2.c
	mipsel-linux-gnu-gcc -o ./write_long_mips_le $(CFLAGS) ./write_long.c
	mipsel-linux-gnu-gcc -o ./write_test_mips_le $(CFLAGS) ./write_test.c
	mipsel-linux-gnu-gcc -o ./write_test2_mips_le $(CFLAGS) ./write_test2.c
	mips-linux-gnu-gcc -o ./write_long_mips_be $(CFLAGS) ./write_long.c
	mips-linux-gnu-gcc -o ./write_test_mips_be $(CFLAGS) ./write_test.c
	mips-linux-gnu-gcc -o ./write_test2_mips_be $(CFLAGS) ./write_test2.c

entropy: clean_entropy
	gcc -o ./entropy $(CFLAGS) ./entropy.c
	mipsel-linux-gnu-gcc -o ./entropy_mips_le $(CFLAGS) ./entropy.c

wrap_and_win: clean_wrap_and_win
	gcc -o ./wrap_and_win $(CFLAGS) ./wrap_and_win.c
	mipsel-linux-gnu-gcc -o ./wrap_and_win_mips_le $(CFLAGS) ./wrap_and_win.c

flip12: clean_flip12
	gcc -Wall -g -o flip12x flip12.c

target: clean_target
	gcc -Wall -g -o targetx target.c

test_entropy: entropy
	gdb ./entropy

clean: clean_uaf clean_spray_and_pray clean_writes clean_entropy clean_wrap_and_win clean_flip12 clean_target
