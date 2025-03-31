import json
import sys

def load_json_files(base_path, head_path):
    """Load and parse the JSON files."""
    with open(base_path, 'r') as f:
        base_data = json.load(f)
    
    with open(head_path, 'r') as f:
        head_data = json.load(f)
        
    return base_data, head_data

def calculate_percentage_change(old_value, new_value):
    """Calculate percentage change between values."""
    if old_value == 0:
        return "N/A" if new_value == 0 else "+∞%"
    
    change = ((new_value - old_value) / old_value) * 100
    prefix = "+" if change > 0 else ""
    return f"{prefix}{change:.2f}%"

def generate_overview_table(base_data, head_data):
    """Generate the overview table comparing base and head sizes."""
    rows = []
    
    for build_type in ["debug", "release"]:
        crates_key = f"size_crates_{build_type}.json"
        
        if crates_key in base_data and crates_key in head_data:
            base_file_size = base_data[crates_key]["file-size"]
            head_file_size = head_data[crates_key]["file-size"]
            file_size_change = calculate_percentage_change(base_file_size, head_file_size)
            
            base_text_size = base_data[crates_key]["text-section-size"]
            head_text_size = head_data[crates_key]["text-section-size"]
            text_size_change = calculate_percentage_change(base_text_size, head_text_size)
            
            rows.append([
                f"{build_type.capitalize()} Total File Size",
                f"{base_file_size:,} bytes",
                f"{head_file_size:,} bytes",
                file_size_change
            ])
            
            rows.append([
                f"{build_type.capitalize()} Total Text Section Size",
                f"{base_text_size:,} bytes",
                f"{head_text_size:,} bytes",
                text_size_change
            ])

            size_base_crate = None
            size_head_crate = None
            
            for crate in base_data[crates_key]["crates"]:
                if crate["name"] == "wasm":
                    size_base_crate = crate["size"]
                    break
    
            for crate in head_data[crates_key]["crates"]:
                if crate["name"] == "wasm":
                    size_head_crate = crate["size"]
                    break
            
            assert size_base_crate is not None
            assert size_head_crate is not None

            size_crate_change = calculate_percentage_change(size_base_crate, size_head_crate)

            rows.append([
                f"{build_type.capitalize()} Crate Size",
                f"{size_base_crate:,} bytes",
                f"{size_head_crate:,} bytes",
                size_crate_change
            ])

    table = "| Metric | Base | Head | Change |\n"
    table += "|--------|------|------|--------|\n"
    
    for row in rows:
        table += f"| {row[0]} | {row[1]} | {row[2]} | {row[3]} |\n"
    
    return table

def get_top_wasm_functions(base_data, head_data, count=10):
    """Get the top N functions by size."""
    # Extract all wasm functions from head
    head_funcs_debug = []
    head_funcs_release = []
    
    for build_type in ["debug", "release"]:
        funcs_key = f"size_funcs_{build_type}.json"
        
        if funcs_key in head_data:
            for func in head_data[funcs_key]["functions"]:
                if "crate" in func and func["crate"] == "wasm":
                    if build_type == "debug":
                        head_funcs_debug.append(func)
                    else:
                        head_funcs_release.append(func)
    
    # Get corresponding functions from base (if they exist)
    base_funcs_by_name_debug = {}
    base_funcs_by_name_release = {}
    
    for build_type in ["debug", "release"]:
        funcs_key = f"size_funcs_{build_type}.json"
        
        if funcs_key in base_data:
            for func in base_data[funcs_key]["functions"]:
                if "crate" in func and func["crate"] == "wasm":
                    if build_type == "debug":
                        base_funcs_by_name_debug[func["name"]] = func
                    else:
                        base_funcs_by_name_release[func["name"]] = func
    
    # Sort by size (descending) and take top N
    head_funcs_debug.sort(key=lambda x: x["size"], reverse=True)
    head_funcs_release.sort(key=lambda x: x["size"], reverse=True)
    
    top_debug = head_funcs_debug[:count]
    top_release = head_funcs_release[:count]
    
    debug_table = generate_functions_table(top_debug, base_funcs_by_name_debug, "Debug")
    release_table = generate_functions_table(top_release, base_funcs_by_name_release, "Release")
    
    return debug_table, release_table

def generate_functions_table(top_functions, base_functions, build_type):
    """Generate a markdown table for the top functions."""
    table = f"### {build_type} Build\n\n"
    table += "| Function | Size (Base) | Size (Head) | Change |\n"
    table += "|----------|-------------|-------------|--------|\n"
    
    for func in top_functions:
        func_name = func["name"]
        head_size = func["size"]
        
        if func_name in base_functions:
            base_size = base_functions[func_name]["size"]
            change = calculate_percentage_change(base_size, head_size)
            table += f"| `{func_name}` | {base_size:,} bytes | {head_size:,} bytes | {change} |\n"
        else:
            table += f"| `{func_name}` | N/A | {head_size:,} bytes | New |\n"
    
    return table

def generate_report(base_data, head_data):
    """Generate the full markdown report."""
    overview_table = generate_overview_table(base_data, head_data)
    debug_funcs_table, release_funcs_table = get_top_wasm_functions(base_data, head_data)
    
    report = ""
    report += "# Size Report\n\n"
    report += "## Overview\n\n"
    report += overview_table + "\n\n"
    report += "## Top 10 Functions by Size (Debug)\n\n"
    report += debug_funcs_table + "\n\n"
    report += "## Top 10 Functions by Size (Release)\n\n"
    report += release_funcs_table + "\n\n"
    
    return report

def main():
    if len(sys.argv) != 3:
        print("Usage: python3 compare_sizes.py base_file.json head_file.json")
        sys.exit(1)
    
    base_path = sys.argv[1]
    head_path = sys.argv[2]
    
    base_data, head_data = load_json_files(base_path, head_path)
    report = generate_report(base_data, head_data)
    
    print(report)        

if __name__ == "__main__":
    main()