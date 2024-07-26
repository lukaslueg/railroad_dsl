A small DSL to generate syntax diagrams using [this library](https://github.com/lukaslueg/railroad).

[![Crates.io Version](https://img.shields.io/crates/v/railroad_dsl.svg)](https://crates.io/crates/railroad_dsl)
[![Build status](https://github.com/lukaslueg/railroad_dsl/actions/workflows/check.yml/badge.svg)](https://github.com/lukaslueg/railroad_dsl/actions/workflows/check.yml)

* `{...}` is a horizontal stack of connected elements
* `[...]` is a vertical sequence of connected elements
* `<...>` is a choice of multiple options, exactly one of which has to be picked
* `"foobar"` is a terminal
* `'foobar'` is a non-terminal
* `` `foobar` `` is a comment
* `...?` is an optional element
* `...*...` is a repeated element
* `!` is the empty element

Quotes (and backslashes) can be escaped using backslashes.

For example:

```raw
{["CONSTRAINT" "name"]?,
 <["PRIMARY" "KEY" <!, "ASC", "DESC"> 'conflict-clause' <!, "AUTOINCREMENT">],
  ["NOT" "NULL" 'conflict-clause'],
  ["UNIQUE" 'conflict-clause'],
  ["CHECK" "(" 'expr' ")"],
  ["DEFAULT" <'signed-number', 'literal-value', ["(" 'expr' ")"]>],
  ["COLLATE" "collation-name"],
  'foreign-key-clause'>}
```

![diagram for constraint syntax](https://raw.githubusercontent.com/lukaslueg/railroad_dsl/master/examples/column_constraint.jpeg)

```raw
{[`create-table-stmt` "CREATE" <!, <"TEMP", "TEMPORARY">#`Table will be dropped when connection closes`> "TABLE"],
 [[["IF" "NOT" "EXISTS"]#`If table exists, do nothing`]? [[["schema-name" "."]#`...in a foreign database`]? "table-name"]#`The table's name`],
 [<["(" ['column-def'*","]#`One or more column-definitions` [!*[['table-constraint' ","]#`primary key and stuff`]]#`Zero or more table-constraints` ")" <!, ["WITHOUT" "ROWID"]>],
   ["AS" 'select-stmt']#`Create table definition and content directly from a query`>]}
```

![diagram for create-table syntax](https://raw.githubusercontent.com/lukaslueg/railroad_dsl/master/examples/create_table_stmt.jpeg)


Run `cargo run --example example_diagrams` for more examples.

---

```raw
A small DSL to generate syntax-diagrams.

If no input files are given, act as a pipe from stdin to stdout. Otherwise, process each input file into an output file with the file extension replaced

Usage: railroad [OPTIONS] [INPUTS]...

Arguments:
  [INPUTS]...
          

Options:
      --css <CSS>
          Alternative CSS file

      --format <FORMAT>
          Output format
          
          [default: svg]
          [possible values: svg, png]

      --max-width <MAX_WIDTH>
          Maximum width of the final image

      --max-height <MAX_HEIGHT>
          Maximum height of the final image

      --theme <THEME>
          Theme to use
          
          [default: light]
          [possible values: light, dark]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
