#!/usr/bin/env python3
import os
import subprocess
from pathlib import Path
import shutil
import json

LLVM_ROOT = Path('/usr/local/llvm-nyuzi')
NYUZI_ROOT = Path('../NyuziProcessor').resolve()
LIB_DIR = NYUZI_ROOT / 'software' / 'libs'
OUT_DIR = Path('out').resolve()

CLANG = LLVM_ROOT / 'bin' / 'clang'
ELF2HEX = LLVM_ROOT / 'bin' / 'elf2hex'
EMU = NYUZI_ROOT / 'bin' / 'emulator'
VERILATOR = NYUZI_ROOT / 'bin' / 'verilator_model'

INCLUDES = [
    "-I", LIB_DIR / "libc" / "include",
    "-I", LIB_DIR / "libos",
    "-I", LIB_DIR / "libos" / "bare-metal"
]
CRT = [
    LIB_DIR / "libc" / "libc.a",
    LIB_DIR / "compiler-rt" / "compiler-rt.a",
    LIB_DIR / "libos" / "crt0-bare.o",
    LIB_DIR / "libos" / "libos-bare.a",
]
CXXFLAGS = ['-std=c++11', '-O3']

def sh(cli, **kwargs):
    cli = [str(arg) if isinstance(arg, Path) else arg for arg in cli]
    proc = subprocess.run(cli, **kwargs, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if proc.returncode:
        print("FAILED:")
        print(proc.stdout.decode('utf-8'))
    proc.check_returncode()

def _build_rust_variant(bench, variant, features):
    print("Building Rust benchmark:", bench, variant)
    os.chdir('rust_nyuzi_staticlib')
    env = dict(os.environ)
    assert 'RUSTFLAGS' not in env
    env['RUSTFLAGS'] = '--cfg benchmark="{}" --cfg variant="{}"'.format(bench, variant)
    sh(['xargo', 'build', '--target=nyuzi-elf-none', '--release', '--features', features],
       env=env)
    CARGO_OUTPUT = 'target/nyuzi-elf-none/release/librust_nyuzi_staticlib.a'
    archive = OUT_DIR / (bench  + '_' + variant + '.a')
    shutil.copy(str(CARGO_OUTPUT), str(archive))
    os.chdir('..')
    return _build_harness(bench, variant, archive)

def _build_cxx_variant(bench, variant, source_file, threads):
    defines = ['-DBENCH_' + bench.upper(), '-DVARIANT_' + variant.upper()]
    if threads:
        defines.append('-DUSE_THREADS')
    print("Building C++ benchmark:", bench, variant, "(threads)" if threads else "")
    obj = OUT_DIR / (bench + '_' + variant + '.o')
    sh([CLANG, source_file, *CXXFLAGS, *INCLUDES, *defines, '-c', '-o', obj])
    return _build_harness(bench, variant, obj, threads)

def _build_harness(bench, variant, bench_obj, threads=False):
    defines = ['-DBENCH_NAME=' + bench, '-DBENCH_VARIANT=' + variant]
    if threads:
        defines.append('-DUSE_THREADS')
        variant += '_threads'
    elf_path = OUT_DIR / (bench + '_' + variant + '.elf')
    hex_path = elf_path.with_suffix('.hex')
    sh([CLANG, bench_obj, 'harness.cpp', *CXXFLAGS, *INCLUDES, *CRT, *defines,
        '-o', elf_path])
    sh([ELF2HEX, elf_path, '-o', hex_path])
    return (bench, variant, hex_path, bench_obj, elf_path)

def build_rust(name, features):
    yield _build_rust_variant(name, 'scalar', features)
    yield _build_rust_variant(name, 'spmd', features)

def build_cxx(name, source_file):
    for variant in ('scalar', 'spmd', 'intrin'):
        for threads in (False, True):
            yield _build_cxx_variant(name, variant, source_file, threads)

def build_all():
    return [
        *build_cxx('hash', 'hash/hash.cpp'),
        *build_cxx('mandelbrot', 'mandelbrot/mandelbrot.cpp'),
        *build_rust('fib_iter', features='link_fib'),
        *build_rust('fib_rec', features='link_fib'),
        *build_rust('nbody', features='link_nbody'),
        *build_rust('fwt', features='link_fwt'),
        *build_rust('fwt_nodivmod', features='link_fwt'),
    ]

def run(hex_path):
    proc = subprocess.run(
        [str(VERILATOR), '+bin=' + str(hex_path), '+randseed=0x12345678'],
        stdout=subprocess.PIPE, check=True,
    )
    prefix = 'elapsed:'
    for line in proc.stdout.decode('utf-8').split('\n'):
        if line.startswith(prefix):
            return int(line[len(prefix):].strip())
    raise Exception("did not find cycle count in harness output")

def main():
    assert Path.cwd() == Path(__file__).resolve().parent
    shutil.rmtree(str(OUT_DIR))
    OUT_DIR.mkdir()
    benchmarks = build_all()
    results = []
    for (bench, variant, hex_path, obj_path, elf_path) in benchmarks:
        print("Running:", bench, variant)
        BENCH_RUNS = 3
        cycles_measurements = []
        for _ in range(BENCH_RUNS):
            cycles_measurements.append(run(hex_path))
        obj_size = obj_path.stat().st_size
        exe_size = elf_path.stat().st_size
        results.append({
            'bench': bench,
            'variant': variant,
            'cycles': cycles_measurements,
            'obj_size': obj_size,
            'exe_size': exe_size
        })
    with open('bench-data.json', 'w') as f:
        json.dump(results, f, indent=2, sort_keys=True)
        f.write('\n')

if __name__ == '__main__':
    main()
