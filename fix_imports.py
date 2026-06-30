import os

files = [
    r"src/spatial_id/collection/expr/binary/arith/tests.rs",
    r"src/spatial_id/collection/expr/binary/set/tests.rs",
    r"src/spatial_id/collection/expr/unary/fill/tests.rs",
    r"src/spatial_id/collection/expr/unary/level/tests.rs",
    r"src/spatial_id/collection/expr/unary/shift/tests.rs",
    r"src/spatial_id/collection/expr/unary/spread/tests.rs",
    r"src/spatial_id/collection/expr/unary/stretch/tests.rs"
]

for path in files:
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    if "SpatialIdCollection" not in content:
        content = "use crate::SpatialIdCollection;\n" + content
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)

print("Done")
