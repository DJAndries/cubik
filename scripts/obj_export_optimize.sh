#!/bin/sh

if [ "$#" != "2" ]; then
	echo "usage: $0 <blend filename> <obj output filename>" 1>&2
	exit 1
fi

blender $1 --background --python $(dirname $0)/obj_export_optimize.py -- $2
