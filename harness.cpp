#include <stdio.h>

extern "C" {
void fib_scalar();
void fib_spmd();
void nbody_scalar();
void nbody_spmd();
void mandelbrot_scalar();
void mandelbrot_spmd();
void mandelbrot_intrinsics();
void hash_scalar();
void hash_spmd();
void hash_intrinsics();
void fwt_scalar();
void fwt_spmd();
void fwt_nomod_scalar();
void fwt_nomod_spmd();
}

namespace {
struct Benchmark {
  const char *name;
  void (*scalar)();
  void (*spmd)();
  void (*intrinsics)();
};

Benchmark BENCHMARKS[] = {
    {"fib", fib_scalar, fib_spmd, nullptr},
    {"nbody", nbody_scalar, nbody_spmd, nullptr},
    {"mandelbrot", mandelbrot_scalar, mandelbrot_spmd, mandelbrot_intrinsics},
    {"hash", hash_scalar, hash_spmd, hash_intrinsics},
    {"fwt", fwt_scalar, fwt_spmd, nullptr},
    {"fwt_nomod", fwt_nomod_scalar, fwt_nomod_spmd, nullptr}};

int measure_cycles(void (*f)()) {
  int t0 = __builtin_nyuzi_read_control_reg(6);
  f();
  return __builtin_nyuzi_read_control_reg(6) - t0;
}

extern "C" {
void nop() {}
}

void run_benchmark(Benchmark &B) {
  int elapsed_scalar = measure_cycles(B.scalar);
  int elapsed_spmd = measure_cycles(B.spmd);
  int elapsed_intrin = 0;
  printf("bench:%s, %d, %d,", B.name, elapsed_scalar, elapsed_spmd);
  if (B.intrinsics) {
    elapsed_intrin = measure_cycles(B.intrinsics);
    printf(" %d", elapsed_intrin);
  }
  printf("\n");
}
}

int main() {
  for (auto &B : BENCHMARKS) {
    run_benchmark(B);
  }
  return 0;
}
