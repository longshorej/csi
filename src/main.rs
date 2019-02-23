use std::{collections, env, fs, io, io::prelude::*, path, process};

trait Context {
    fn active(&self, name: &str) -> bool;
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
        // @TODO only allow CSI_ vars to come from local
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

fn compile_directive<C: Context>(
    context: &mut C,
    directive: String,
    content: &mut Vec<char>,
) -> io::Result<String> {
    let let_html = directive.starts_with("let html");
    let let_raw = directive.starts_with("let raw");
    let var_html = directive.starts_with("var html ");
    let var_raw = directive.starts_with("var raw ");
    let include_html = directive.starts_with("include html ");
    let include_raw = directive.starts_with("include raw ");
    let require_html = directive.starts_with("require html ");
    let require_raw = directive.starts_with("require raw ");
    let wrapped_html = directive.starts_with("wrapped html");
    let wrapped_raw = directive.starts_with("wrapped raw");

    if var_html || var_raw || let_html || let_raw {
        let (_, var) = directive.split_at(if var_html { 9 } else { 8 });

        match context.load_var(var) {
            Some(value) => {
                if var_html || let_html {
                    Ok(escape_text(&value))
                } else {
                    Ok(value)
                }
            }

            None if let_html || let_raw => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("cannot find variable: {}", var),
            )),

            None => Ok("".to_string()),
        }
    } else if include_html
        || include_raw
        || require_html
        || require_raw
        || wrapped_html
        || wrapped_raw
    {
        let (_, path) = directive.split_at(if include_html || require_html || wrapped_html {
            13
        } else {
            12
        });

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

            if wrapped_html || wrapped_raw {
                // @TODO a name
                context.add_var("CSI_WRAPPED".to_string(), content.iter().collect());
                println!("set var!");
            }

            let result = compile_path(context, &path, require_html || require_raw);

            if wrapped_html || wrapped_raw {
                context.remove_var("CSI_WRAPPED");
            }

            if result.is_ok() {
                content.clear();
            }

            result.map(|value| {
                if include_html {
                    escape_text(&value)
                } else {
                    value
                }
            })
        }
    } else {
        Ok("".to_string())
    }
}

fn compile_path<C: Context>(
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
    let result = compile(context, &raw_content);
    context.remove_active(path.to_str().unwrap_or_default());

    env::set_current_dir(original_dir)?;

    result
}

fn compile<C: Context>(context: &mut C, content: &str) -> io::Result<String> {
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

                for d in compile_directive(context, directive, &mut content)?.chars() {
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

fn run(root: &path::Path, src: &path::Path, dest: &path::Path) -> io::Result<()> {
    let mut context = FsContext::new();

    let root = path::Path::new(root);
    let src = path::Path::new(src);
    let dest = path::Path::new(dest);

    if src.is_dir() {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                run(root, &path, dest)?;
            } else {
                // @TODO skip _

                let dest = dest.join(
                    path.strip_prefix(root)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
                );

                let src_compiled = compile_path(&mut context, &path.canonicalize()?, true)?;

                if let Some(dest_parent) = dest.parent() {
                    fs::create_dir_all(dest_parent)?;
                }

                let mut dest_file = fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(dest)?;

                dest_file.write_all(src_compiled.as_bytes())?;
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() == 3 {
        let root = path::Path::new(&args[1]);
        let src = path::Path::new(&args[1]);
        let dest = path::Path::new(&args[2]);

        run(&root, &src, &dest)?;

        process::exit(0)
    } else {
        println!(
            "{} <src-dir> <dest-dir>",
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
