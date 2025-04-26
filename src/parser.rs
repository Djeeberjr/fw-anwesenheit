use regex::Regex;

/// Parses the output of PM3 finds the read IDs
/// Example input: `[+] UID.... 3112B710`
pub fn parse_line(line: &str) -> Option<String> {
    let regex = Regex::new(r"(?m)^\[\+\] UID.... (.*)$").unwrap();
    let result = regex.captures(line);

    result.map(|c| c.get(1).unwrap().as_str().to_owned())
}
