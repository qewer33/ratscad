; OpenSCAD-flavoured highlights running against the C parser's parse tree.
; The C parser doesn't understand OpenSCAD's `module`, `function`, or `$fn`-style
; vars, so we use #match? predicates on (identifier) nodes to colour OpenSCAD
; keywords and builtins by their textual name. `union` is also a C reserved
; word so we capture its literal token to override.

(comment) @comment

(string_literal) @string

(number_literal) @number

; Default for all identifiers; specific matches below override.
(identifier) @variable

; OpenSCAD keywords (subset that survives the C parser as identifiers).
((identifier) @keyword
  (#match? @keyword "^(module|function|let|if|else|for|each|true|false|undef|use|include|echo|assert)$"))

; OpenSCAD builtin primitives, transforms, boolean ops, and stdlib functions.
((identifier) @function.call
  (#match? @function.call "^(cube|sphere|cylinder|polyhedron|polygon|square|circle|text|linear_extrude|rotate_extrude|hull|minkowski|translate|rotate|scale|mirror|color|multmatrix|resize|offset|difference|intersection|render|projection|surface|import|children|len|str|chr|ord|search|abs|sign|sin|cos|tan|asin|acos|atan|atan2|sqrt|pow|exp|ln|log|min|max|floor|ceil|round|norm|cross|concat|rands|version|version_num|parent_module|fill)$"))

; `union(...)` collides with the C `union` keyword token — capture the literal
; so it still gets the function colour.
"union" @function.call
