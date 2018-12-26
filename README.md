# Jinja template generator

> Hacked together for private usage

Create a Jinja2 template and a YAML with the default values.
Useful to generate ansible config templates based on existing config files.

## Usage

    cargo run input.conf var_prefix

A `template.j2` file will be created with all the config variables uncommented and values replaced with variables.

A supporting `variables.yml` will be created with the default values based on the input configuration file.

## Known issues

- Code must be horrifying (my first try with rust)
- No multiline array support (generates broken YAML)