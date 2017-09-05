#include <stdio.h>

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
