# CSI

[![Crates.io](https://img.shields.io/crates/v/csi.svg?style=flat-square)](https://crates.io/crates/csi)
[![Crates.io](https://img.shields.io/crates/d/csi.svg?style=flat-square)](https://crates.io/crates/csi)
[![Travis CI](https://img.shields.io/travis/longshorej/csi.svg?style=flat-square)](https://travis-ci.org/longshorej/csi)

CSI -- client-side includes - is a tool for processing a directory of text files. It allows you to define files that include other files, and substitute variables.

The primary use-case is to make building pure HTML sites a little bit easier.

## Features

* Process a directory of files
* Include files in other files
* Set variables to arbitrary values
* Include environment and scoped variable values in files

## Install

You can use `cargo` to install the tool.

```bash
cargo install csi
```

## Usage

```bash
csi <input-directory> <output-directory> [--ext <extension>]...
```

For example, given the following command:

```bash
csi src target/dist --ext .html --ext .css
```

CSI will walk the input directory and process all files.

* If a file begins with `_` it will be skipped.
* If a file ends in `.html` or `.css`, it will be processed and written to the output directory.
* If a file doesn't, it will be copied verbatim to the output directory.

Note that the directory structure is perserved.

## Directives

CSI's features are provided via *directives* which are simple statements in your files.  Directives are enclosed in `[]` -- e.g. `[include raw my-file.html]`. A directive is a space separated list of arguments.

> To simplify its behavior, CSI does not trim white-space. If a directive cannot be parsed, the program exits with a failure.

### Variable substitution

You can use the `var` or `opt` directives to substitute environment variables into the file. `var` indicates that the variable is required, while `opt` indicates it is optional.

If a variable isn't defined, and `opt` is used, the directive will evaluate to the empty string.

#### Format

```
var <format> <variable>
```

```
opt <format> <variable>
```

Format may be `html` or `raw`. If `html`, it will be escaped for use in an HTML document. If `raw`, it will be substituted directly.

*When using `raw`, be sure that you're not subjecting yourself to XSS attacks.*

```html
<p>Hello [var html MY_VAR]</p>
```

### Set

You can set a variable for use in the current file or included ones.

#### Format

```
set <name> <value>
```

#### Example

```html
[set name John]
[include raw _template.html]
```

### Stash

Stash will take all of the current evaluated content and place it into the specified variable. Content after the stash directive is excluded. This is useful for defining some content in a file, and then evaluating it in the context of a template that renders variables. Also known as the decorator pattern.

#### Format

```
stash <variable>
```


#### Example

```html
<p>This is my content</p>

[stash content][require _layout.html]
```

### Includes

You can include files in other files. If a file includes a file that includes itself, that include will be ignored to break the cycle.

File paths are relative to the file that is being processed.

If the file doesn't exist, the directive will evaluate to the empty string.

#### Format

```
include <format> [path]
```

Format may be `html` or `raw`. If `html`, it will be escaped for use in an HTML document. If `raw`, it will be substituted directly.

*When using `raw`, be sure that you're not subjecting yourself to XSS attacks.*

#### Example

```html
<pre>[include raw /etc/passwd]</pre>
```

### Requires

You can require files in other files. If a file includes a file that includes itself, the program will exit with a failure.

File paths are relative to the file that is being processed.

If the file doesn't exist, the program will exit with a failure.

#### Format

```
require <format> [path]
```

Format may be `html` or `raw`. If `html`, it will be escaped for use in an HTML document. If `raw`, it will be substituted directly.

*When using `raw`, be sure that you're not subjecting yourself to XSS attacks.*

#### Example

```html
<pre>[require raw /etc/passwd]</pre>
```

## Changelog

### 1.0.0 - 2019-02-24

Initial release.
