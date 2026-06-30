import os
import re

files = [
    r"src/spatial_id/collection/expr/binary/set/tests.rs",
    r"src/spatial_id/collection/expr/unary/fill/tests.rs",
    r"src/spatial_id/collection/expr/unary/level/tests.rs",
    r"src/spatial_id/collection/expr/unary/shift/tests.rs",
    r"src/spatial_id/collection/expr/unary/spread/tests.rs",
    r"src/spatial_id/collection/expr/unary/stretch/tests.rs"
]

methods = [
    "union_with", "intersection_with", "difference", "symmetric_difference", "mask",
    "fill_default", "level_f", "level_x", "level_y", "level_f_with", "level_x_with", "level_y_with",
    "shift_f", "shift_x", "shift_y",
    "spread_axes_with", "spread", "spread_with", "spread_x", "spread_x_with", "spread_y", "spread_y_with", "spread_f", "spread_f_with", "spread_xyz", "spread_xyz_with",
    "stretch_f", "stretch_x", "stretch_y", "stretch_f_with", "stretch_x_with", "stretch_y_with"
]

for path in files:
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    # Remove traits imports
    for trait in ["SetOps", "FillOps", "LevelOps", "ShiftOps", "SpreadOps", "StretchOps"]:
        content = content.replace(trait + ",", "")
        content = content.replace(trait, "")

    # Replace `.method(...)` with `.into_query().method(...).run()`
    # We will do this carefully using regex
    for method in methods:
        # Match `.method(`
        # We need to find the matching closing parenthesis and insert `.run()` after it.
        pattern = re.compile(r"(\w+)\s*\.\s*" + method + r"\s*\(")
        
        while True:
            m = pattern.search(content)
            if not m:
                break
            
            # Find matching closing paren
            start = m.end() - 1
            depth = 0
            end = -1
            for i in range(start, len(content)):
                if content[i] == '(':
                    depth += 1
                elif content[i] == ')':
                    depth -= 1
                    if depth == 0:
                        end = i
                        break
            
            if end != -1:
                # Replace the match
                prefix = content[:m.start()]
                var_name = m.group(1)
                
                args = content[start+1:end]
                # For binary ops, if args starts with `&`, remove it (e.g., `&other` -> `other`)
                # Wait, if we remove `&`, `other` might need to be cloned if it's used later, 
                # but in most tests it's just `&b_table()` or `&b`. Let's just remove `&` for `b`.
                if method in ["union_with", "intersection_with", "difference", "symmetric_difference", "mask"]:
                    args = re.sub(r"^&\s*", "", args)
                
                replacement = f"{var_name}.into_query().{method}({args}).run()"
                suffix = content[end+1:]
                
                content = prefix + replacement + suffix
            else:
                break

    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

print("Done")
