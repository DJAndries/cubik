- adding clipping to the ground to fix camera if moving towards ground
- skybox
- refactor some funcs to use mpl, such as wavefront, for sharing state and cleaner code
- jumping
- add circle/sphere sat collision/collisionobj, add SAT poly-circle + cicle-circle
- performance consideration: separate quadoctrees for polygons and triangles/terrain
- minor performance consideration: check for duplicating axes for sat poly-poly, negative and positive
w to tell positive or negative
- fix starting direction of camera (mouse problem)
