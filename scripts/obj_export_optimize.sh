#!/bin/sh

if [ "$#" lt "2" ]; then
	echo "usage: $0 <map name>" 1>&2
	exit 1
fi

blender $1.blend --background --python $(dirname $0)/obj_export_optimize.py --$1.obj
