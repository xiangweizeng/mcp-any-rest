# ZML Quickstart (Current Implementation)

> All comments in examples are in English, complying with workspace rules.

## Tools
- `zml`: Unified ZML CLI (supports listing modules and compiling to JSON)

### Usage
- List modules (using default configuration directory):
```bash
cargo run --bin zml -- list
```
- List modules (specifying custom ZML directory):
```bash
cargo run --bin zml -- list --dir custom/zml/dir
```
- Read from file and output JSON (using default configuration directory):
```bash
cargo run --bin zml -- compile -i config/zml/project.zml > project.json
```
- Read from standard input and output JSON:
```bash
cat config/zml/project.zml | cargo run --bin zml -- compile -- > project.json
```
- Pretty print output (indentation):
```bash
cargo run --bin zml -- compile -i config/zml/project.zml --pretty
```

### Parameter Description
- `-i, --input FILE`: Input ZML file path; reads from STDIN if not provided
- `-p, --pretty`: Pretty print output as indented JSON
- `-c, --config-dir DIR`: Configuration directory path (Default: automatically detects `config/` under program directory)
- `-d, --dir DIR`: ZML directory path (Default: `zml/` under configuration directory)
- `-o, --out DIR`: Preset output directory (Default: `presets/` under configuration directory)

## Example: Compiling a Module
```zml
module project {
    version: "1.0.0"
    description: "Project service"
    enabled: true
    access_level: public

    type project_user {
        id: integer                 // User ID
        account: string             // User account
    }

    method get_project_list {
        description: "Get project list"
        http_method: GET
        uri: "projects"
        access_level: public
        rate_limit: 50/60

        params {
            page: integer = 1       // Default 1
            limit: integer = 20     // Default 20
        }

        response: object {
            page: integer
            total: integer
        }
    }
}
```

Compile:
```bash
cargo run --bin zml -- compile -i config/zml/project.zml --pretty
```

## FAQ
- Empty Input: When reading from STDIN, the tool will error and exit if the content is empty.
- Syntax Errors: The compiler will output error messages; please fix them according to `docs/ZML_SPECIFICATION.md`.
- Templates and Inheritance: Currently only parsed, not involved in compilation output; please refer to the limitations section in the specification document.

## Further Reading
- Specification (Chinese): `docs/ZML_SPECIFICATION.md`
- Specification (English): `docs/ZML_SPECIFICATION.en.md`
- Nesting and Reference Validation: `docs/ZML_NESTING_REFERENCE_VALIDATION*.md`
