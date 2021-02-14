import bpy
import sys

bpy.ops.object.select_all(action='DESELECT')
for collection in bpy.data.collections:
	if collection.name.endswith('_merge'):
		print(f'Merging {collection.name}...')
		bpy.ops.object.select_same_collection(collection=collection.name)
		if len(bpy.context.selected_objects) == 0:
			continue
		bpy.context.view_layer.objects.active = bpy.context.selected_objects[0]
		bpy.ops.object.join()
		bpy.context.active_object.name = collection.name[:-6]
		bpy.ops.object.select_all(action='DESELECT')

bpy.ops.export_scene.obj(filepath=sys.argv[-1], path_mode='RELATIVE')
