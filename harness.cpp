#include <stdio.h>

/*extern "C" {
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
  const char *variant;
  void (*func)();
};

Benchmark BENCHMARKS[] = {
    {"fib", fib_scalar, fib_spmd, nullptr},
    {"nbody", nbody_scalar, nbody_spmd, nullptr},
    {"mandelbrot", mandelbrot_scalar, mandelbrot_spmd, mandelbrot_intrinsics},
    {"hash", hash_scalar, hash_spmd, hash_intrinsics},
    {"fwt", fwt_scalar, fwt_spmd, nullptr},
    {"fwt_nomod", fwt_nomod_scalar, fwt_nomod_spmd, nullptr}};
*/

#if !defined(BENCH_NAME) || !defined(BENCH_VARIANT)
#error "Missing BENCH_NAME / BENCH_VARIANT"
#endif


#define CONCAT_(x, y) x ## _ ## y
#define CONCAT(x, y) CONCAT_(x, y)
#define BENCH_FUNC CONCAT(BENCH_NAME, BENCH_VARIANT)

extern "C" {
  void BENCH_FUNC();
}

int main() {
  int t0 = __builtin_nyuzi_read_control_reg(6);
  BENCH_FUNC();
  int elapsed = __builtin_nyuzi_read_control_reg(6) - t0;
  printf("elapsed:%d\n", elapsed);
  return 0;
}
