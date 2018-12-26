use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(PartialEq)]
enum ConfigLevel {
    Root,
    Section,
    SubSection,
}

#[derive(PartialEq)]
enum VariableType {
    Boolean,
    String,
    Transparent, // no escaping is applied, numbers for example
}

fn normalized_line(line: &str) -> String {
    let mut line = line.trim();

    if line.starts_with('#') {
        line = line[1..].trim_start()
    }

    line.to_owned()
}

fn is_config_line(line: &str) -> bool {
    if line.len() < 3 {
        return false;
    }

    if !line.contains(" = ") {
        return false;
    }

    let spl: Vec<&str> = line.trim_left().split(" = ").collect();

    if spl.len() != 2 {
        return false;
    }

    if spl[0].contains(' ') {
        return false;
    }

    true
}

fn type_of_value(value: &str) -> VariableType {
    if value == "true" || value == "false" {
        return VariableType::Boolean;
    }

    if value.starts_with('"') {
        return VariableType::String;
    }

    VariableType::Transparent
}

fn parse_config_line(line: &str) -> (String, String, VariableType) {
    // TODO &str would be better
    let spl: Vec<&str> = line.split(" = ").collect();
    if spl.len() != 2 {
        panic!("Could not parse config line");
    }

    let key = spl[0].to_owned();
    let value = spl[1].to_owned();
    let variable_type = type_of_value(&value);

    (key, value, variable_type)
}

fn get_line_prefix(level: &ConfigLevel) -> String {
    if level == &ConfigLevel::Root {
        return String::from("");
    }

    String::from("  ")
}

fn get_variable_name(
    prefix: &str,
    level: &ConfigLevel,
    section: &str,
    sub_section: &str,
    name: &str,
) -> String {
    let mut var = String::from("");
    var.push_str(&prefix);
    var.push('_');

    if level != &ConfigLevel::Root {
        var.push_str(&section.replace('-', "_"));
        var.push('_');
        if level == &ConfigLevel::SubSection {
            var.push_str(&sub_section.replace('-', "_"));
            var.push('_');
        }
    }

    var.push_str(&name.replace('-', "_"));

    var
}

fn escape_config_value(value: &str, variable_type: &VariableType) -> String {
    if variable_type == &VariableType::Boolean {
        let mut result = String::new();
        result.push('"');
        result.push_str(value);
        result.push('"');

        return result;
    }

    String::from(value)
}

fn variable_line(
    prefix: &str,
    name: &str,
    variable_name: &str,
    variable_type: &VariableType,
) -> String {
    let mut result = String::new();

    result.push_str(prefix);
    result.push_str(name);

    result.push_str(" = ");

    if variable_type == &VariableType::String {
        result.push_str("\"");
    }

    result.push_str("{{ ");
    result.push_str(variable_name);
    result.push_str(" }}");

    if variable_type == &VariableType::String {
        result.push_str("\"");
    }

    result
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("Usage: {} input.conf prefix", &args[0])
    }

    let filename = &args[1];
    let config_prefix = &args[2];

    let file = File::open(filename).expect("cannot open file");
    let file = BufReader::new(file);

    let mut result = String::new();
    let mut variables = String::new();

    let mut level = ConfigLevel::Root;
    let mut section = String::new();
    let mut sub_section = String::new();
    for original_line in file.lines().filter_map(|result| result.ok()) {
        let line = normalized_line(&original_line);
        let mut resulting_line = original_line;

        // Detect sections
        if line.len() > 4 && &line[0..2] == "[[" {
            if level == ConfigLevel::Root {
                panic!("Found subsection at root level! Line: '{}'", line);
            }

            level = ConfigLevel::SubSection;
            sub_section = String::from(&line[2..line.len() - 2]);

            variables.push_str(&"\n# [[");
            variables.push_str(&sub_section);
            variables.push_str(&"]]\n");
        } else if line.len() > 2 && &line[0..1] == "[" {
            level = ConfigLevel::Section;
            section = String::from(&line[1..line.len() - 1]);

            variables.push_str(&"\n# [");
            variables.push_str(&section);
            variables.push_str(&"]\n");
        } else if is_config_line(&line) {
            let (name, value, variable_type) = parse_config_line(&line);
            //println!("Found config:\n  name: {}\n  value: {}\n", name, value)

            let variable_name =
                get_variable_name(config_prefix, &level, &section, &sub_section, &name);

            variables.push_str(&variable_name);
            variables.push_str(&": ");
            variables.push_str(&escape_config_value(&value, &variable_type));
            variables.push_str(&"\n");

            let prefix = get_line_prefix(&level);

            resulting_line = variable_line(&prefix, &name, &variable_name, &variable_type);
        }

        // add content to target file
        result.push_str(&resulting_line);
        result.push_str(&"\n");
    }

    fs::write("template.j2", result).expect("Unable to write file");
    fs::write("variables.yml", variables).expect("Unable to write file");

    //println!("{}", result)
}
