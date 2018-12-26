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

fn parse_config_line(line: &str) -> (String, String) {
    // TODO &str would be better
    let spl: Vec<&str> = line.split(" = ").collect();
    if spl.len() != 2 {
        panic!("Could not parse config line");
    }

    (spl[0].to_owned(), spl[1].to_owned())
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
            let (name, value) = parse_config_line(&line);
            //println!("Found config:\n  name: {}\n  value: {}\n", name, value)

            let variable_name =
                get_variable_name(config_prefix, &level, &section, &sub_section, &name);

            variables.push_str(&variable_name);
            variables.push_str(&": ");
            variables.push_str(&value);
            variables.push_str(&"\n");

            let prefix = get_line_prefix(&level);

            resulting_line = prefix + &name + " = {{ " + &variable_name + " }}";
        }

        // add content to target file
        result.push_str(&resulting_line);
        result.push_str(&"\n");
    }

    fs::write("template.j2", result).expect("Unable to write file");
    fs::write("variables.yml", variables).expect("Unable to write file");

    //println!("{}", result)
}
