#!/usr/bin/env python3

import json
import csv
from collections import defaultdict

def add_speedup(res, *, of, over):
    t_old = float(res[over])
    t_new = float(res[of])
    name = 'speedup ' + of + ' over ' + over
    res[name] = round(t_old / t_new, 2)
    if name not in PROPERTIES:
        PROPERTIES.append(name)

def add_size_increase(res, *, kind, baseline):
    size_baseline = float(res[baseline + '_' + kind + 'size'])
    size_spmd = float(res['spmd_' + kind + 'size'])
    name = kind + ' size increase over ' + baseline
    res[name] = round(size_spmd / size_baseline, 2)
    if name not in PROPERTIES:
        PROPERTIES.append(name)

PROPERTIES = "scalar,spmd,intrin,scalar_objsize,scalar_exesize,spmd_objsize,spmd_exesize,intrin_objsize,intrin_exesize".split(',')

def main():
    # read measurements
    with open('bench-data.json') as f:
        raw_data = json.load(f)
    results = defaultdict(dict)
    for data_point in raw_data:
        bench = data_point['bench']
        variant = data_point['variant']
        assert variant in PROPERTIES
        cycle_measurements = data_point['cycles']
        assert len(set(cycle_measurements)) == 1, "TODO varying cycle counts??"
        results[bench][variant] = cycle_measurements[0]
        results[bench][variant + '_objsize'] = data_point['obj_size']
        results[bench][variant + '_exesize'] = data_point['exe_size']

    # compute speedups
    for res in results.values():
        add_speedup(res, of='spmd', over='scalar')
        if 'intrin' in res:
            add_speedup(res, of='intrin', over='scalar')
            add_speedup(res, of='spmd', over='intrin')
    # compute size increases
    for res in results.values():
        for kind in ('obj', 'exe'):
            add_size_increase(res, kind=kind, baseline='scalar')
            if 'intrin' in res:
                add_size_increase(res, kind=kind, baseline='intrin')

    # -> sorta-pretty csv
    csv_data = [[None] + PROPERTIES]
    for bench, res in sorted(results.items()):
        csv_data.append([bench] + [res.get(prop) for prop in PROPERTIES])
    with open('bench-data.csv', 'w', newline='') as f:
        csvwriter = csv.writer(f)
        for csv_row in csv_data:
            csvwriter.writerow(csv_row)

if __name__ == '__main__':
    main()
