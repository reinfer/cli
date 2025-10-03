#!/usr/bin/env python3
"""
Preprocess OpenAPI spec to fix type issues before Rust client generation.
Specifically fixes EntityDefNew.id and FieldChoiceNewApi.id typed as literal None.
"""

import json
import sys
from pathlib import Path
from typing import Dict, Any

def fix_none_typed_ids(spec: Dict[str, Any]) -> Dict[str, Any]:
    """Fix EntityDefNew.id and FieldChoiceNewApi.id typed as literal None."""
    
    if 'components' not in spec or 'schemas' not in spec['components']:
        return spec
    
    schemas = spec['components']['schemas']
    
    # Fix EntityDefNew.id
    if 'EntityDefNew' in schemas:
        entity_def = schemas['EntityDefNew']
        if 'properties' in entity_def and 'id' in entity_def['properties']:
            id_prop = entity_def['properties']['id']
            # Check if it's typed as None or null
            if (isinstance(id_prop, dict) and 
                (id_prop.get('type') == 'null' or 
                 id_prop.get('enum') == [None] or
                 str(id_prop).lower() == 'none')):
                # Fix it to be an optional string
                entity_def['properties']['id'] = {
                    'type': 'string',
                    'nullable': True
                }
                print(f"✓ Fixed EntityDefNew.id from literal None to nullable string")
    
    # Fix FieldChoiceNewApi.id  
    if 'FieldChoiceNewApi' in schemas:
        field_choice = schemas['FieldChoiceNewApi']
        if 'properties' in field_choice and 'id' in field_choice['properties']:
            id_prop = field_choice['properties']['id']
            # Check if it's typed as None or null
            if (isinstance(id_prop, dict) and 
                (id_prop.get('type') == 'null' or 
                 id_prop.get('enum') == [None] or
                 str(id_prop).lower() == 'none')):
                # Fix it to be an optional string
                field_choice['properties']['id'] = {
                    'type': 'string', 
                    'nullable': True
                }
                print(f"✓ Fixed FieldChoiceNewApi.id from literal None to nullable string")
    
    return spec

def fix_invalid_schemas(spec: Dict[str, Any]) -> Dict[str, Any]:
    """Fix other common invalid schema issues."""
    
    if 'components' not in spec or 'schemas' not in spec['components']:
        return spec
    
    schemas = spec['components']['schemas']
    
    # Scan all schemas for literal None types
    for schema_name, schema in schemas.items():
        if isinstance(schema, dict) and 'properties' in schema:
            for prop_name, prop in schema['properties'].items():
                if isinstance(prop, dict):
                    # Fix any property typed as literal None
                    if (prop.get('type') == 'null' or 
                        prop.get('enum') == [None] or
                        str(prop).lower() == 'none'):
                        schema['properties'][prop_name] = {
                            'type': 'string',
                            'nullable': True
                        }
                        print(f"✓ Fixed {schema_name}.{prop_name} from literal None to nullable string")
    
    return spec

def validate_schemas(spec: Dict[str, Any]) -> bool:
    """Validate that schemas don't contain invalid types."""
    
    if 'components' not in spec or 'schemas' not in spec['components']:
        return True
    
    schemas = spec['components']['schemas']
    valid = True
    
    for schema_name, schema in schemas.items():
        if isinstance(schema, dict) and 'properties' in schema:
            for prop_name, prop in schema['properties'].items():
                if isinstance(prop, dict):
                    # Check for invalid None types
                    if (prop.get('type') == 'null' or 
                        prop.get('enum') == [None]):
                        print(f"✗ Invalid type in {schema_name}.{prop_name}: {prop}")
                        valid = False
    
    return valid

def preprocess_spec(input_file: str, output_file: str) -> None:
    """Preprocess OpenAPI spec file to fix type issues."""
    
    print(f"▶ Preprocessing OpenAPI spec: {input_file}")
    
    with open(input_file, 'r') as f:
        spec = json.load(f)
    
    print(f"▶ Validating input spec...")
    if validate_schemas(spec):
        print("✓ Input spec appears valid")
    else:
        print("⚠ Input spec has invalid schemas - attempting fixes...")
    
    # Apply fixes
    spec = fix_none_typed_ids(spec)
    spec = fix_invalid_schemas(spec)
    
    print(f"▶ Validating output spec...")
    if validate_schemas(spec):
        print("✓ Output spec is valid")
    else:
        print("✗ Output spec still has issues!")
        return
    
    # Write processed spec
    with open(output_file, 'w') as f:
        json.dump(spec, f, indent=2)
    
    print(f"✔ Preprocessed spec written to: {output_file}")

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print("Usage: preprocess-spec.py <input.json> <output.json>")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    
    if not Path(input_file).exists():
        print(f"Error: Input file {input_file} does not exist")
        sys.exit(1)
    
    preprocess_spec(input_file, output_file)
