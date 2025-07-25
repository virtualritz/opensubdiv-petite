# Creased Cube to STEP Example

This example demonstrates how to:
1. Create a cube mesh with creased edges using OpenSubdiv
2. Apply Catmull-Clark subdivision
3. Export the resulting subdivision surface as a STEP file

## Features

- Creates a unit cube with 8 vertices and 6 quad faces
- Applies crease values of 3.0 to three edges sharing vertex 0 (origin corner)
- Performs 3 levels of Catmull-Clark subdivision
- Converts the subdivided mesh to truck geometry
- Exports the result as a STEP file

## Building and Running

From the example directory:
```bash
cargo run --release
```

This will generate `creased_cube.step` in the current directory.

## Expected Output

The program will output:
- The subdivision level used (3)
- Initial and final vertex counts
- Initial and final face counts
- The edges that were creased

## Viewing the Result

The generated STEP file can be viewed in any CAD software that supports STEP format, such as:
- FreeCAD
- OpenSCAD (with STEP import plugin)
- CAD Assistant
- Any professional CAD software

## Customization

You can modify:
- `max_level`: Change the subdivision level (higher = smoother but more vertices)
- `crease_values`: Adjust the sharpness of creases (0 = no crease, infinity = sharp edge)
- `creased_edges`: Add or remove edges to crease
- Initial geometry: Replace the cube with any other mesh