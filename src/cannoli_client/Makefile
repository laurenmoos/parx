# note: need apt -y install gcc-mipsel-linux-gnu
CFLAGS = -O0 -g -static -fno-pic

all: clean
	gcc -o ./binaries/wrap_and_win $(CFLAGS) ./binaries/wrap_and_win.c
	mipsel-linux-gnu-gcc -o ./binaries/wrap_and_win_mips $(CFLAGS) ./binaries/wrap_and_win.c
	nm ./binaries/wrap_and_win_mips > ./symbols.txt

symbols:
	rm -f ./symbols.txt
	nm ../examples/spray_and_pray_mips_le > symbols.txt

run:
	cat ./data/input_32.bin | ../qemu/build/qemu-mipsel \
        ./binaries/wrap_and_win_mips

run_cannoli:
	cat ./data/input_32.bin | ../qemu/build/qemu-mipsel \
        -cannoli ~/Documents/harden/mvp/target/release/libmvp.so \
        ./binaries/wrap_and_win_mips

test_write_le:
	../qemu/build/qemu-mipsel \
        -cannoli ~/Documents/harden/mvp/target/release/libmvp.so \
        ../examples/write_long_mips_le

test_write_be:
	../qemu/build/qemu-mips \
        -cannoli ~/Documents/harden/mvp/target/release/libmvp.so \
        ../examples/write_long_mips_be

debug:
	cat ./data/input_32.bin | ../../qemu-clean/build/qemu-mipsel \
        -g 9999 ./binaries/wrap_and_win_mips

debug_attach:
	gdb-multiarch ./binaries/wrap_and_win_mips -ex "target remote :9999"

run_s:
	cargo +nightly run --features "mips" 

test:
	cargo +nightly test --features "mips" -- --nocapture

clean:
	rm -f ./binaries/wrap_and_win_mips
	rm -f ./binaries/wrap_and_win
