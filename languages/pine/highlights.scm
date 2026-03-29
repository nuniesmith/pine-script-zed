; highlights.scm — Pine Script v6 (tree-sitter, aspirational)
; ==========================================================
; This query file targets a future tree-sitter-pine grammar.
; Node names are best-guess placeholders until a grammar exists.

; ---------------------------------------------------------------------------
; Keywords
; ---------------------------------------------------------------------------

; Conditionals
(if) @keyword.conditional
(else) @keyword.conditional
(switch) @keyword.conditional

; Loops
(for) @keyword.repeat
(while) @keyword.repeat

; Storage / declarations
(var) @keyword.storage
(varip) @keyword.storage

; Control flow
(return) @keyword.return
(break) @keyword.control
(continue) @keyword.control

; Module system
(import) @keyword.import
(export) @keyword.export

; Object-oriented
(type) @keyword.type
(method) @keyword.function

; Logical operators (keyword form)
(and) @keyword.operator
(or) @keyword.operator
(not) @keyword.operator

; ---------------------------------------------------------------------------
; Types (built-in)
; ---------------------------------------------------------------------------

(int) @type.builtin
(float) @type.builtin
(bool) @type.builtin
(string) @type.builtin
(color) @type.builtin
(series) @type.builtin
(array) @type.builtin
(matrix) @type.builtin
(map) @type.builtin

; ---------------------------------------------------------------------------
; Constants (built-in)
; ---------------------------------------------------------------------------

(na) @constant.builtin
(true) @constant.builtin.boolean
(false) @constant.builtin.boolean

; ---------------------------------------------------------------------------
; Literals
; ---------------------------------------------------------------------------

(string) @string
((int_literal) @number)
((float_literal) @number.float)

; ---------------------------------------------------------------------------
; Functions and calls
; ---------------------------------------------------------------------------

(function_definition name: (identifier) @function)

((identifier) @function.builtin
 (#match? @function.builtin "^(ta|request|strategy|math|color|array|matrix|map|plot|str|timeframe|ticker|syminfo|log)$"))

(call_expression function: (identifier) @function.call)

; ---------------------------------------------------------------------------
; Variables
; ---------------------------------------------------------------------------

(identifier) @variable

; ---------------------------------------------------------------------------
; Operators
; ---------------------------------------------------------------------------

(binary_expression operator: _ @operator)
(unary_expression operator: _ @operator)

; ---------------------------------------------------------------------------
; Punctuation — brackets
; ---------------------------------------------------------------------------

"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket

; ---------------------------------------------------------------------------
; Punctuation — delimiters
; ---------------------------------------------------------------------------

"." @punctuation.delimiter
"," @punctuation.delimiter
":" @punctuation.delimiter
"?" @punctuation.delimiter

; ---------------------------------------------------------------------------
; Punctuation — special / assignment
; ---------------------------------------------------------------------------

"=>" @punctuation.special
":=" @punctuation.special
"=" @punctuation.special

; ---------------------------------------------------------------------------
; Comments
; ---------------------------------------------------------------------------

(comment) @comment
