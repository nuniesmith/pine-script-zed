; Pine Script highlights for Zed
; Targets the kvarenzn/tree-sitter-pine grammar.

; Variables (generic; specific cases below override these)
(identifier) @variable

(attribute attribute: (identifier) @property)

; Constants (SCREAMING_CASE)
((identifier) @constant
  (#match? @constant "^[A-Z][A-Z_0-9]*$"))

; Built-in namespaces / constants
((identifier) @constant.builtin
  (#any-of? @constant.builtin
   "adjustment" "alert" "array" "barmerge" "box" "chart" "color" "currency"
   "dayofweek" "display" "dividends" "earnings" "extend" "font" "format"
   "hline" "input" "label" "line" "linefill" "location" "log" "map" "math"
   "matrix" "na" "order" "plot" "polyline" "position" "request" "runtime"
   "scale" "session" "shape" "size" "splits" "str" "strategy" "syminfo"
   "ta" "table" "text" "ticker" "timeframe" "xloc" "yloc"))

; Built-in series / variables
((identifier) @variable.special
  (#any-of? @variable.special
   "bar_index" "barstate" "close" "dayofmonth" "high" "hl2" "hlc3" "hlcc4"
   "hour" "last_bar_index" "last_bar_time" "low" "minute" "month" "ohlc4"
   "open" "second" "time" "time_close" "time_tradingday" "timenow" "volume"
   "weekofyear" "year"))

; Calls
(call function: (identifier) @function)
(call function: (attribute attribute: (identifier) @function))
(template_function name: (attribute attribute: (identifier) @function))

; Built-in functions
((call function: (identifier) @function.builtin)
 (#any-of? @function.builtin
  "alert" "alertcondition" "barcolor" "bgcolor" "bool" "box" "color"
  "dayofmonth" "dayofweek" "fill" "fixnan" "float" "hline" "hour" "indicator"
  "input" "int" "label" "library" "line" "linefill" "max_bars_back" "minute"
  "month" "na" "nz" "plot" "plotarrow" "plotbar" "plotcandle" "plotchar"
  "plotshape" "second" "strategy" "string" "time" "time_close" "timestamp"
  "weekofyear" "year"))

; Constructors (`Type.new(...)`)
((call function: (attribute attribute: (identifier) @constructor))
 (#eq? @constructor "new"))

; Declarations
(function_declaration_statement function: (identifier) @function)
(function_declaration_statement method: (identifier) @function)
(function_declaration_statement argument: (identifier) @variable)
(keyword_argument key: (identifier) @property)

; Types
(type_definition_statement name: (identifier) @type)
(base_type (identifier) @type)
((base_type (identifier) @type.builtin)
 (#any-of? @type.builtin
  "array" "bool" "box" "chart" "color" "float" "int" "label" "line"
  "linefill" "map" "matrix" "polyline" "series" "simple" "string" "table"))

; Imports
(import path: (import_path) @string.special)
(import alias: (identifier) @variable)

; Keywords
[
  "export"
  (break)
  (continue)
] @keyword

["method"] @keyword
["var" "varip"] @keyword
["simple" "const" "series"] @keyword
["import" "as"] @keyword
["if" "else" "switch"] @keyword
["for" "while" "in" "to" "by"] @keyword
["type"] @keyword

; Operators
[
  "=" ":=" "==" "!=" ">=" "<=" "+" "+=" "-" "-=" "*" "*=" "/" "/=" "%" "%=" "=>"
] @operator
["and" "or" "not"] @keyword
(comparison_operation ["<" ">"] @operator)
(conditional_expression ["?" ":"] @operator)

; Literals
[(true) (false)] @boolean
(integer) @number
(float) @number
(color) @string.special
(string) @string
(escape_sequence) @string.escape

; Punctuation
["," "."] @punctuation.delimiter
["[" "]"] @punctuation.bracket
(parenthesized_expression ["(" ")"] @punctuation.bracket)

; Comments and annotations (`//@version`, `//@param`, …)
(comment) @comment
(annotations) @keyword
