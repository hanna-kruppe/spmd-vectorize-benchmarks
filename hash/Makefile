COMPILER_DIR=/usr/local/llvm-nyuzi/bin
LIBDIR=../../NyuziProcessor/software/libs

CXX=$(COMPILER_DIR)/clang++
CFLAGS=-O3 -I$(LIBDIR)/libc/include -I$(LIBDIR)/libos -Wall -W -std=c++11

all:
	$(CXX) $(CFLAGS) -c hash.cpp -o bench.o

clean:
	rm -f bench.o
