use itertools::Itertools;

/// Returns static string containing `ident_level` number of spaces, if `ident_level > 100` panics. #TODO
pub fn indent(ident_level: usize) -> &'static str {
    // Yep, this is string of 100 spaces
    let much_space = "                                                                                                    ";
    &much_space[0..ident_level]
}

/// Replaces leading whitespace on every line with `ident_level` number of spaces
pub fn set_indent<T: AsRef<str>>(str: T, ident_level: usize) -> String {
    let str: &str = str.as_ref();
    let spliter;
    if str.contains("\r") {
        spliter = "\r\n";
    } else {
        spliter = "\n";
    }
    str.split(spliter)
        .map(|line| format!("{}{}", indent(ident_level), line.trim_start()))
        .join("\n")
}
