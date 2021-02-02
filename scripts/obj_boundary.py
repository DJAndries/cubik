#!/usr/bin/env python3

import sys

if len(sys.argv) != 2:
	print(f'usage: {sys.argv[0]} <obj path>', file=sys.stderr)
	sys.exit(1)

min = [sys.float_info.max, sys.float_info.max, sys.float_info.max]
max = [sys.float_info.min, sys.float_info.min, sys.float_info.min]

f = open(sys.argv[1], 'r')
for line in f:
	if not line.startswith('v '):
		continue
	vertex = line.split(' ')[1:]
	for i in range(0, 3):
		vertex_f = float(vertex[i])
		if vertex_f < min[i]:
			min[i] = vertex_f
		if vertex_f > max[i]:
			max[i] = vertex_f

print(f'Min: {min}')
print(f'Max: {max}')
