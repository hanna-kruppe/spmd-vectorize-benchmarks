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

PROPERTIES = "scalar,scalar_threads,spmd,spmd_threads,intrin,intrin_threads".split(',')

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
        assert len(set(cycle_measurements)) == 1, "TODO do something with varying cycle counts"
        results[bench][variant] = cycle_measurements[0]
        if variant in {'scalar', 'spmd', 'intrin'}:
            objkey = variant + '_objsize'
            results[bench][objkey] = data_point['obj_size']
            if objkey not in PROPERTIES:
                PROPERTIES.append(objkey)
            exekey = variant + '_exesize'
            results[bench][exekey] = data_point['exe_size']
            if exekey not in PROPERTIES:
                PROPERTIES.append(exekey)
    for bench in results.keys():
        obj_size = data_point['obj_size']

    # compute speedups and size reductions
    for res in results.values():
        add_speedup(res, of='spmd', over='scalar')
        if 'scalar_threads' in res:
            add_speedup(res, of='spmd_threads', over='scalar_threads')
        if 'intrin' in res:
            add_speedup(res, of='intrin', over='scalar')
            add_speedup(res, of='spmd', over='intrin')
        if 'intrin_threads' in res:
            add_speedup(res, of='intrin_threads', over='scalar_threads')
            add_speedup(res, of='spmd_threads', over='intrin_threads')
        res['obj size increase over scalar'] = round(float(res['spmd_objsize']) / float(res['scalar_objsize']), 2)
        res['exe size increase over scalar'] = round(float(res['spmd_exesize']) / float(res['scalar_exesize']), 2)
        if 'intrin_objsize' in res:
            res['obj size increase over intrin'] = round(float(res['spmd_objsize']) / float(res['intrin_objsize']), 2)
            res['exe size increase over intrin'] = round(float(res['spmd_exesize']) / float(res['intrin_exesize']), 2)

    PROPERTIES.append('obj size increase over scalar')
    PROPERTIES.append('obj size increase over intrin')
    PROPERTIES.append('exe size increase over scalar')
    PROPERTIES.append('exe size increase over intrin')

    # -> sorta-pretty csv
    headers = sorted(results)
    csv_data = [[None] + headers]
    for prop in PROPERTIES:
        csv_data.append([prop] + [results[res_key].get(prop) for res_key in headers])
    with open('bench-data.csv', 'w', newline='') as f:
        csvwriter = csv.writer(f)
        for csv_row in csv_data:
            csvwriter.writerow(csv_row)

if __name__ == '__main__':
    main()
