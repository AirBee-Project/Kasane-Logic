import os
import re

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

    content = content.replace(".into_query()", ".clone().into_query()")

    for method in ["add", "sub", "mul", "union_with", "intersection_with", "difference", "symmetric_difference", "mask"]:
        # Match `.method(var` or `.method(var()`
        # \1 will be `var` or `var()`
        pattern = r"\." + method + r"\(\s*([a-zA-Z0-9_]+(\(\))?)"
        content = re.sub(pattern, r"." + method + r"(\1.clone()", content)

    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

print("Done")
