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
TODO: actually skip _ files

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

#### Context Variables

In addition to environment variables, the following context variables are available for use:

| Name                  | Value                                                              |
| --------------------- | ------------------------------------------------------------------ |
| CSI_SURROUNDED        |                                                                    |

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

### Wraps

You can indicate that a file should be wrapped with the contents of another file. This requires a directive to be specified in both files.

#### Surrounded File

In the file that is to be surrounded (i.e. included in another one), use the `wrapped` directive with the name of the file. This should normally be the last directive in a file, as only the contents that have been evaluated so far will be surrounded.

```
Here is some content.

[surround raw layout.html]
```

#### Surrounding File

In the file that will surround, typically a layout or template file, use a `let` or `var` directive to include the content. CSI will ensure that the `CSI_WRAPPED` variable is available.

```
<html>
<body>
[let raw CSI_WRAPPED]
</body>
</html>
```
