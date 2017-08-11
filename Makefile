LLVM_ROOT := /usr/local/llvm-nyuzi
LIB_DIR := ../NyuziProcessor/software/libs
INCLUDES := -I $(LIB_DIR)/libc/include -I $(LIB_DIR)/libos
CRT := $(LIB_DIR)/libc/libc.a $(LIB_DIR)/compiler-rt/compiler-rt.a \
       $(LIB_DIR)/libos/crt0-bare.o $(LIB_DIR)/libos/libos-bare.a

CFLAGS := -std=c++11
CLANG := $(LLVM_ROOT)/bin/clang
ELF2HEX := $(LLVM_ROOT)/bin/elf2hex
EMU := ../NyuziProcessor/bin/emulator
VERILATOR := ../NyuziProcessor/bin/verilator_model

BENCH_ARTIFACTS := _rust_aggregate/bench.a mandelbrot/bench.o hash/bench.o

run: benchmarks
	$(ELF2HEX) program.elf -o program.hex
	$(VERILATOR) +bin=program.hex +randseed=0xc0fefe | tee bench.log
	./analyse.py

benchmarks:
	cd _rust_aggregate && $(MAKE)
	cd mandelbrot && $(MAKE)
	cd hash && $(MAKE)
	$(CLANG) $(CFLAGS) $(INCLUDES) harness.cpp $(CRT) $(BENCH_ARTIFACTS) -o program.elf

clean:
	cd _rust_aggregate && $(MAKE) clean
	rm -f program.{elf,hex}
