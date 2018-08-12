A small DSL to generate syntax diagrams using [this library](https://github.com/lukaslueg/railroad).

**[Some examples](https://htmlpreview.github.io/?https://github.com/lukaslueg/railroad_dsl/blob/master/examples/example_diagrams.html)**

--

* `{...}` is a horizontal stack of connected elements
* `[...]` is a vertical sequence of connected elements
* `<...>` is a choice of multiple options, exactly one of which has to be picked
* `"foobar"` is a terminal
* `'foobar'` is a non-terminal
* `` `foobar` `` is a comment
* `...?` is an optional element
* `...*...` is a repeated element
* `!` is the empty element

For example:

```
{["CONSTRAINT", "name"]?,
 <["PRIMARY", "KEY", <!, "ASC", "DESC">, 'conflict-clause', <!, "AUTOINCREMENT">],
  ["NOT", "NULL", 'conflict-clause'],
  ["UNIQUE", 'conflict-clause'],
  ["CHECK", "(", 'expr', ")"],
  ["DEFAULT", <'signed-number', 'literal-value', ["(", 'expr', ")"]>],
  ["COLLATE", "collation-name"],
  'foreign-key-clause'>}
```

![diagram for constraint syntax](https://raw.githubusercontent.com/lukaslueg/railroad_dsl/master/examples/column_constraint.jpeg)