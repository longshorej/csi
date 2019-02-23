# CSI

CSI -- client-side includes - is a tool for processing a directory of text files. It allows you to define files that include other files, and substitute variables.

The primary use-case is to make building pure HTML sites a little bit easier.

## Features

* Process a directory of files
* Include files in other files
* Include environment and context variable values in files

## Usage

```bash
csi <input-directory> <output-directory>
```

CSI will walk the input directory and process all files. If a file begins with `_` it will be skipped.

## Status

Currently being implemented.

TODO: white-list extensions to avoid binary files

## Syntax

CSI directives are enclosed in `[]` -- e.g. `[include my-file.html]`. A directive is a space separated list of arguments.

To simplify its behavior, CSI does not trim white-space. If a directive cannot be parsed, it is evaluated to an empty string.

### HTML variable substitution (escapes)

You can use the `var` or `let` directives to substitute environment variables into the file. `var` indicates that the variable is optional, while `let` indicates it is required.

If a variable isn't defined, and `var` is used, the directive will evaluate to the empty string.

#### Format

```
var <format> <variable>
```

```
let <format> <variable>
```

Format may be `html` or `raw`. If `html`, it will be escaped for use in an HTML document. If `raw`, it will be substituted directly.

*When using `raw`, be sure that you're not subjecting yourself to XSS attacks.*

```html
<p>Hello [var html MY_VAR]</p>
```

### Set

You can set a variable for use in the current file or includes ones.

#### Format

```
set <name> <value>
```

### Stash

Stash will take the current evaluated content and place it into the specified variable.

#### Format

```
stash <variable>
```

### Includes

You can include files in other files. If a file includes a file that includes itself, that include will be ignored to break the cycle.

If the file doesn't exist, the directive will evaluate to the empty string.

#### Format

```
include <format> [path]
```

Format may be `html` or `raw`. If `html`, it will be escaped for use in an HTML document. If `raw`, it will be substituted directly.

*When using `raw`, be sure that you're not subjecting yourself to XSS attacks.*

```html
<pre>[include raw /etc/passwd]</pre>
```

### Requires

You can require files in other files. If a file includes a file that includes itself, the program will exit with a failure.

If the file doesn't exist, the program will exit with a failure.

#### Format

```
require <format> [path]
```

Format may be `html` or `raw`. If `html`, it will be escaped for use in an HTML document. If `raw`, it will be substituted directly.

*When using `raw`, be sure that you're not subjecting yourself to XSS attacks.*

```html
<pre>[require raw /etc/passwd]</pre>
```
