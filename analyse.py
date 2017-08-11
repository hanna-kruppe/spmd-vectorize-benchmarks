#!/usr/bin/env python3

import csv
from collections import namedtuple

Result = namedtuple('Result', 'name scalar spmd intrin')

def print_result(res):
    print("=====", res.name)
    print("scalar cycles:", res.scalar)
    print("spmd   cycles:", res.spmd)
    if res.intrin is not None:
        print("intrin cycles:", res.intrin)
    print("speedup:   spmd over scalar:", float(res.scalar) / float(res.spmd))
    if res.intrin is not None:
        print("speedup: intrin over scalar:", float(res.scalar) / float(res.intrin))
        print("speedup:   spmd over intrin:", float(res.intrin) / float(res.spmd))
    print()

def main():
    with open('bench.log', newline='') as f:
        lines = [line[len('bench:'):] for line in f if line.startswith('bench:')]
        for row in csv.reader(lines):
            name, scalar, spmd, intrin_opt = row
            intrin = int(intrin_opt) if intrin_opt else None
            res = Result(name, int(scalar), int(spmd), intrin)
            print_result(res)

if __name__ == '__main__':
    main()
