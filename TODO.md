# big features
- materials
- sounds
- weapon
- npc
- net com
- ui
- water

# small features
- add circle/sphere sat collision/collisionobj, add SAT poly-circle + cicle-circle
- add translation matrix to collisionobj

# fixes
- fix starting direction of camera (mouse problem)
- allow objs with no textures
- less jittery camera
- see todo comments

# optimizations
- refactor some funcs to be impl funcs, such as wavefront, for sharing state and cleaner code
- only add objects with terrain prefix to octree as triangle
- performance consideration: separate quadoctrees for polygons and triangles/terrain
- minor performance consideration: check for duplicating axes for sat poly-poly, negative and positive
w to tell positive or negative
- warnings
