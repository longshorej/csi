use std::{collections, env, fs, io, io::prelude::*, path, process};

trait Context {
    fn active(&self, name: &str) -> bool;
    fn export_vars(&self) -> collections::HashMap<String, String>;
    fn replace_vars(&mut self, vars: collections::HashMap<String, String>);
    fn load_file(&self, path: &path::Path, required: bool) -> io::Result<String>;
    fn load_var(&self, name: &str) -> Option<String>;
    fn add_var(&mut self, name: String, value: String);
    fn add_active(&mut self, name: &str);
    fn remove_active(&mut self, name: &str);
    fn remove_var(&mut self, name: &str);
}

struct FsContext {
    active: collections::HashSet<String>,
    vars: collections::HashMap<String, String>,
}

impl FsContext {
    fn new() -> Self {
        let active = collections::HashSet::new();
        let vars = collections::HashMap::new();

        Self { active, vars }
    }
}

impl Context for FsContext {
    fn export_vars(&self) -> collections::HashMap<String, String> {
        self.vars.clone()
    }

    fn replace_vars(&mut self, vars: collections::HashMap<String, String>) {
        self.vars = vars;
    }

    fn add_var(&mut self, name: String, value: String) {
        self.vars.insert(name, value);
    }

    fn remove_var(&mut self, name: &str) {
        self.vars.remove(name);
    }

    fn active(&self, name: &str) -> bool {
        self.active.contains(name)
    }

    fn load_file(&self, path: &path::Path, required: bool) -> io::Result<String> {
        if path.exists() {
            let mut file = fs::File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            Ok(contents)
        } else if required {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("cannot read file: {}", path.to_str().unwrap_or_default()),
            ))
        } else {
            Ok("".to_string())
        }
    }

    fn load_var(&self, name: &str) -> Option<String> {
        match self.vars.get(name) {
            Some(value) => Some(value.to_string()),
            None => env::var(name).ok(),
        }
    }

    fn add_active(&mut self, name: &str) {
        self.active.insert(name.to_string());
    }

    fn remove_active(&mut self, name: &str) {
        self.active.remove(name);
    }
}

fn process_directive<C: Context>(
    context: &mut C,
    directive: String,
    content: &mut Vec<char>,
) -> io::Result<String> {
    let var_html = directive.starts_with("var html");
    let var_raw = directive.starts_with("var raw");
    let opt_html = directive.starts_with("opt html ");
    let opt_raw = directive.starts_with("opt raw ");
    let include_html = directive.starts_with("include html ");
    let include_raw = directive.starts_with("include raw ");
    let require_html = directive.starts_with("require html ");
    let require_raw = directive.starts_with("require raw ");
    let set = directive.starts_with("set ");
    let stash = directive.starts_with("stash ");

    if set {
        let (_, entry) = directive.split_at(4);

        match entry.find(" ") {
            Some(p) if p < entry.len() - 1 => {
                let (name, value) = entry.split_at(p);
                let name = name.to_string();
                let value = value[1..].to_string();

                context.add_var(name, value);

                Ok("".to_string())
            }

            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("invalid set format: {}", entry),
            )),
        }
    } else if stash {
        let (_, var) = directive.split_at(6);
        let c = content.iter().collect::<String>();

        content.clear();
        context.add_var(var.to_string(), c);

        Ok("".to_string())
    } else if opt_html || opt_raw || var_html || var_raw {
        let (_, var) = directive.split_at(if opt_html { 9 } else { 8 });

        match context.load_var(var) {
            Some(value) => {
                if opt_html || var_html {
                    Ok(escape_text(&value))
                } else {
                    Ok(value)
                }
            }

            None if var_html || var_raw => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("cannot find variable: {}", var),
            )),

            None => Ok("".to_string()),
        }
    } else if include_html || include_raw || require_html || require_raw {
        let (_, path) = directive.split_at(if include_html || require_html { 13 } else { 12 });

        if context.active(path) {
            if require_html || require_raw {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("cannot require file due to cycle: {}", path),
                ))
            } else {
                Ok("".to_string())
            }
        } else {
            let original_dir = env::current_dir()?;
            let path = original_dir.join(path);

            let original_vars = context.export_vars();

            let result = process_path(context, &path, require_html || require_raw);

            context.replace_vars(original_vars);

            result.map(|value| {
                if include_html {
                    escape_text(&value)
                } else {
                    value
                }
            })
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("invalid directive: {}", directive),
        ))
    }
}

fn process_path<C: Context>(
    context: &mut C,
    path: &path::Path,
    required: bool,
) -> io::Result<String> {
    let raw_content = context.load_file(&path, required)?;
    let original_dir = env::current_dir()?;

    if let Some(parent) = path.parent() {
        env::set_current_dir(parent)?;
    }

    context.add_active(path.to_str().unwrap_or_default());
    let result = process(context, &raw_content);
    context.remove_active(path.to_str().unwrap_or_default());

    env::set_current_dir(original_dir)?;

    result
}

fn process<C: Context>(context: &mut C, content: &str) -> io::Result<String> {
    let mut chars = content.chars().peekable();

    let mut escaped = false;
    let mut content = Vec::with_capacity(4096);

    while let Some(c) = chars.next() {
        match c {
            _ if escaped => {
                content.push(c);
            }

            '[' => {
                let mut directive = Vec::new();
                let mut escaped = false;

                while let Some(d) = chars.next() {
                    match d {
                        _ if escaped => {
                            directive.push(d);
                            escaped = false;
                        }

                        ']' => {
                            break;
                        }

                        '\\' => match chars.peek() {
                            Some(']') | Some('\\') => {
                                escaped = true;
                            }

                            _ => {
                                directive.push(d);
                            }
                        },

                        _ => {
                            directive.push(d);
                        }
                    }
                }

                let directive: String = directive.iter().collect();

                for d in process_directive(context, directive, &mut content)?.chars() {
                    content.push(d);
                }
            }

            '\\' => match chars.peek() {
                Some('[') | Some('\\') => {
                    escaped = true;
                }

                _ => {
                    content.push(c);
                }
            },

            _ => {
                content.push(c);
            }
        }
    }

    Ok(content.iter().collect())
}

fn run(
    root: &path::Path,
    src: &path::Path,
    dest: &path::Path,
    extensions: &Vec<&str>,
) -> io::Result<()> {
    let mut context = FsContext::new();

    let root = path::Path::new(root);
    let src = path::Path::new(src);
    let dest = path::Path::new(dest);

    if src.is_dir() {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                run(root, &path, dest, extensions)?;
            } else {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if !name.starts_with("_") {
                    let process = extensions.iter().any(|e| name.ends_with(e));

                    let dest = dest.join(
                        path.strip_prefix(root)
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
                    );

                    if let Some(dest_parent) = dest.parent() {
                        fs::create_dir_all(dest_parent)?;
                    }

                    let path = path.canonicalize()?;

                    if process {
                        let src_processed = process_path(&mut context, &path, true)?;

                        let mut dest_file = fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .create(true)
                            .open(dest)?;

                        dest_file.write_all(src_processed.as_bytes())?;
                    } else {
                        fs::copy(path, dest)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() >= 3 {
        let root = path::Path::new(&args[1]);
        let src = path::Path::new(&args[1]);
        let dest = path::Path::new(&args[2]);

        let mut extensions = Vec::new();
        let mut args_iter = args[3..].iter();

        while let Some(a) = args_iter.next() {
            if a == "--ext" {
                match args_iter.next() {
                    Some(e) => {
                        extensions.push(e.as_str());
                    }

                    None => {
                        println!("--ext must be followed by an extension");
                        process::exit(1);
                    }
                }
            } else {
                println!("unknown argument: {}", a);
                process::exit(1);
            }
        }

        run(&root, &src, &dest, &extensions)?;

        process::exit(0)
    } else {
        println!(
            "{} <src-dir> <dest-dir> <name-pattern>",
            args.get(0).map(|s| s.as_str()).unwrap_or("csi")
        );
        process::exit(1);
    }
}

fn escape_text(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '<' => format!("&lt;"),
            '>' => format!("&gt;"),
            '"' => format!("&quot;"),
            '\'' => format!("&apos;"),
            '&' => format!("&amp;"),
            _ => format!("{}", c),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_text_works_empty() {
        assert_eq!(escape_text(""), "")
    }

    #[test]
    fn escape_text_works_basic() {
        assert_eq!(escape_text("this is a test"), "this is a test")
    }

    #[test]
    fn escape_text_works_html() {
        assert_eq!(
            escape_text("<hello world /> \"123\" wow! & stuff' yeah"),
            "&lt;hello world /&gt; &quot;123&quot; wow! &amp; stuff&apos; yeah"
        )
    }
}
