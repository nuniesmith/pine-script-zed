//! Catalog of Pine Script v6 built-in functions, variables, deprecated names,
//! and language keywords.  Used by the linter (validation) and the LSP server
//! (completions / hover).

// ── Data structures ───────────────────────────────────────────────────────────

/// A built-in function (or namespaced method).
#[derive(Debug, Clone)]
pub struct BuiltinFunc {
    pub name: &'static str,
    pub doc: &'static str,
    /// `(param_name, type_hint)` pairs.
    pub params: &'static [(&'static str, &'static str)],
    pub returns: &'static str,
}

/// A built-in variable / constant.
#[derive(Debug, Clone)]
pub struct BuiltinVar {
    pub name: &'static str,
    pub doc: &'static str,
    pub type_hint: &'static str,
}

/// A function name that was valid in earlier Pine versions but is deprecated in
/// v6, together with its modern replacement.
#[derive(Debug, Clone)]
pub struct DeprecatedFunc {
    pub old_name: &'static str,
    pub new_name: &'static str,
    pub message: &'static str,
}

// ── Built-in functions ────────────────────────────────────────────────────────

macro_rules! builtin_fn {
    ($name:expr, $doc:expr, [ $( ($pn:expr, $pt:expr) ),* $(,)? ], $ret:expr) => {
        BuiltinFunc {
            name: $name,
            doc: $doc,
            params: &[ $( ($pn, $pt) ),* ],
            returns: $ret,
        }
    };
}

static BUILTIN_FUNCTIONS: &[BuiltinFunc] = &[
    // ── Script declarations ──────────────────────────────────────────────
    builtin_fn!(
        "indicator",
        "Declare script as an indicator.",
        [
            ("title", "string"),
            ("shorttitle", "string"),
            ("overlay", "bool"),
            ("format", "string"),
            ("precision", "int"),
            ("scale", "scale_type"),
            ("max_bars_back", "int"),
            ("timeframe", "string"),
            ("timeframe_gaps", "bool"),
            ("explicit_plot_zorder", "bool"),
            ("max_lines_count", "int"),
            ("max_labels_count", "int"),
            ("max_boxes_count", "int"),
        ],
        "void"
    ),
    builtin_fn!(
        "strategy",
        "Declare script as a strategy.",
        [
            ("title", "string"),
            ("shorttitle", "string"),
            ("overlay", "bool"),
            ("format", "string"),
            ("precision", "int"),
            ("scale", "scale_type"),
            ("pyramiding", "int"),
            ("calc_on_order_fills", "bool"),
            ("calc_on_every_tick", "bool"),
            ("max_bars_back", "int"),
            ("backtest_fill_limits_assumption", "int"),
            ("default_qty_type", "string"),
            ("default_qty_value", "float"),
            ("initial_capital", "float"),
            ("currency", "string"),
            ("slippage", "int"),
            ("commission_type", "string"),
            ("commission_value", "float"),
            ("process_orders_on_close", "bool"),
            ("close_entries_rule", "string"),
            ("margin_long", "float"),
            ("margin_short", "float"),
            ("max_lines_count", "int"),
            ("max_labels_count", "int"),
            ("max_boxes_count", "int"),
        ],
        "void"
    ),
    builtin_fn!(
        "library",
        "Declare script as a library.",
        [("title", "string"), ("overlay", "bool"),],
        "void"
    ),
    // ── Plotting ─────────────────────────────────────────────────────────
    builtin_fn!(
        "plot",
        "Plot a series of data on the chart.",
        [
            ("series", "series float"),
            ("title", "string"),
            ("color", "color"),
            ("linewidth", "int"),
            ("style", "plot_style"),
            ("trackprice", "bool"),
            ("histbase", "float"),
            ("offset", "int"),
            ("join", "bool"),
            ("editable", "bool"),
            ("show_last", "int"),
            ("display", "int"),
        ],
        "plot"
    ),
    builtin_fn!(
        "plotshape",
        "Plot a shape on the chart.",
        [
            ("series", "series bool"),
            ("title", "string"),
            ("style", "shape_style"),
            ("location", "location"),
            ("color", "color"),
            ("size", "size"),
            ("text", "string"),
            ("textcolor", "color"),
            ("editable", "bool"),
            ("show_last", "int"),
            ("display", "int"),
        ],
        "void"
    ),
    builtin_fn!(
        "plotchar",
        "Plot a character on the chart.",
        [
            ("series", "series bool"),
            ("title", "string"),
            ("char", "string"),
            ("location", "location"),
            ("color", "color"),
            ("size", "size"),
            ("text", "string"),
            ("textcolor", "color"),
            ("editable", "bool"),
            ("show_last", "int"),
            ("display", "int"),
        ],
        "void"
    ),
    builtin_fn!(
        "plotarrow",
        "Plot an arrow on the chart.",
        [
            ("series", "series int/float"),
            ("title", "string"),
            ("colorup", "color"),
            ("colordown", "color"),
            ("minheight", "int"),
            ("maxheight", "int"),
            ("editable", "bool"),
            ("show_last", "int"),
            ("display", "int"),
        ],
        "void"
    ),
    builtin_fn!(
        "plotbar",
        "Plot OHLC bars.",
        [
            ("open", "series float"),
            ("high", "series float"),
            ("low", "series float"),
            ("close", "series float"),
            ("title", "string"),
            ("color", "color"),
            ("editable", "bool"),
            ("show_last", "int"),
            ("display", "int"),
        ],
        "void"
    ),
    builtin_fn!(
        "plotcandle",
        "Plot candlestick bars.",
        [
            ("open", "series float"),
            ("high", "series float"),
            ("low", "series float"),
            ("close", "series float"),
            ("title", "string"),
            ("color", "color"),
            ("wickcolor", "color"),
            ("bordercolor", "color"),
            ("editable", "bool"),
            ("show_last", "int"),
            ("display", "int"),
        ],
        "void"
    ),
    builtin_fn!(
        "bgcolor",
        "Fill the background of the chart with a color.",
        [
            ("color", "series color"),
            ("offset", "int"),
            ("editable", "bool"),
            ("show_last", "int"),
            ("title", "string"),
            ("display", "int"),
        ],
        "void"
    ),
    builtin_fn!(
        "barcolor",
        "Set the color of the chart bars.",
        [
            ("color", "series color"),
            ("offset", "int"),
            ("editable", "bool"),
            ("show_last", "int"),
            ("title", "string"),
        ],
        "void"
    ),
    builtin_fn!(
        "fill",
        "Fill the area between two plots or hlines.",
        [
            ("hline1", "hline/plot"),
            ("hline2", "hline/plot"),
            ("color", "color"),
            ("title", "string"),
            ("editable", "bool"),
            ("fillgaps", "bool"),
            ("display", "int"),
        ],
        "void"
    ),
    builtin_fn!(
        "hline",
        "Render a horizontal line at a fixed price level.",
        [
            ("price", "float"),
            ("title", "string"),
            ("color", "color"),
            ("linestyle", "hline_style"),
            ("linewidth", "int"),
            ("editable", "bool"),
            ("display", "int"),
        ],
        "hline"
    ),
    // ── input.* ──────────────────────────────────────────────────────────
    builtin_fn!(
        "input.int",
        "Add an integer input to the script settings.",
        [
            ("defval", "int"),
            ("title", "string"),
            ("minval", "int"),
            ("maxval", "int"),
            ("step", "int"),
            ("tooltip", "string"),
            ("inline", "string"),
            ("group", "string"),
            ("confirm", "bool"),
        ],
        "int"
    ),
    builtin_fn!(
        "input.float",
        "Add a float input to the script settings.",
        [
            ("defval", "float"),
            ("title", "string"),
            ("minval", "float"),
            ("maxval", "float"),
            ("step", "float"),
            ("tooltip", "string"),
            ("inline", "string"),
            ("group", "string"),
            ("confirm", "bool"),
        ],
        "float"
    ),
    builtin_fn!(
        "input.bool",
        "Add a boolean input to the script settings.",
        [
            ("defval", "bool"),
            ("title", "string"),
            ("tooltip", "string"),
            ("inline", "string"),
            ("group", "string"),
            ("confirm", "bool"),
        ],
        "bool"
    ),
    builtin_fn!(
        "input.string",
        "Add a string input to the script settings.",
        [
            ("defval", "string"),
            ("title", "string"),
            ("options", "string[]"),
            ("tooltip", "string"),
            ("inline", "string"),
            ("group", "string"),
            ("confirm", "bool"),
        ],
        "string"
    ),
    builtin_fn!(
        "input.color",
        "Add a color input to the script settings.",
        [
            ("defval", "color"),
            ("title", "string"),
            ("tooltip", "string"),
            ("inline", "string"),
            ("group", "string"),
            ("confirm", "bool"),
        ],
        "color"
    ),
    builtin_fn!(
        "input.source",
        "Add a source input to the script settings.",
        [
            ("defval", "series float"),
            ("title", "string"),
            ("tooltip", "string"),
            ("inline", "string"),
            ("group", "string"),
        ],
        "series float"
    ),
    builtin_fn!(
        "input.timeframe",
        "Add a timeframe input to the script settings.",
        [
            ("defval", "string"),
            ("title", "string"),
            ("tooltip", "string"),
            ("inline", "string"),
            ("group", "string"),
            ("confirm", "bool"),
        ],
        "string"
    ),
    builtin_fn!(
        "input.symbol",
        "Add a symbol input to the script settings.",
        [
            ("defval", "string"),
            ("title", "string"),
            ("tooltip", "string"),
            ("inline", "string"),
            ("group", "string"),
            ("confirm", "bool"),
        ],
        "string"
    ),
    // ── ta.* ─────────────────────────────────────────────────────────────
    builtin_fn!(
        "ta.sma",
        "Simple moving average.",
        [("source", "series float"), ("length", "int"),],
        "series float"
    ),
    builtin_fn!(
        "ta.ema",
        "Exponential moving average.",
        [("source", "series float"), ("length", "int"),],
        "series float"
    ),
    builtin_fn!(
        "ta.rsi",
        "Relative strength index.",
        [("source", "series float"), ("length", "int"),],
        "series float"
    ),
    builtin_fn!(
        "ta.macd",
        "Moving average convergence/divergence.",
        [
            ("source", "series float"),
            ("fastlen", "int"),
            ("slowlen", "int"),
            ("siglen", "int"),
        ],
        "[series float, series float, series float]"
    ),
    builtin_fn!(
        "ta.atr",
        "Average true range.",
        [("length", "int"),],
        "series float"
    ),
    builtin_fn!(
        "ta.crossover",
        "True when `source1` crosses over `source2`.",
        [("source1", "series float"), ("source2", "series float"),],
        "series bool"
    ),
    builtin_fn!(
        "ta.crossunder",
        "True when `source1` crosses under `source2`.",
        [("source1", "series float"), ("source2", "series float"),],
        "series bool"
    ),
    builtin_fn!(
        "ta.highest",
        "Highest value for a given number of bars back.",
        [("source", "series float"), ("length", "int"),],
        "series float"
    ),
    builtin_fn!(
        "ta.lowest",
        "Lowest value for a given number of bars back.",
        [("source", "series float"), ("length", "int"),],
        "series float"
    ),
    builtin_fn!(
        "ta.stoch",
        "Stochastic oscillator.",
        [
            ("source", "series float"),
            ("high", "series float"),
            ("low", "series float"),
            ("length", "int"),
        ],
        "series float"
    ),
    builtin_fn!(
        "ta.bb",
        "Bollinger Bands.",
        [
            ("source", "series float"),
            ("length", "int"),
            ("mult", "float"),
        ],
        "[series float, series float, series float]"
    ),
    builtin_fn!(
        "ta.vwap",
        "Volume-weighted average price.",
        [("source", "series float"),],
        "series float"
    ),
    builtin_fn!(
        "ta.pivothigh",
        "Pivot high.",
        [
            ("source", "series float"),
            ("leftbars", "int"),
            ("rightbars", "int"),
        ],
        "series float"
    ),
    builtin_fn!(
        "ta.pivotlow",
        "Pivot low.",
        [
            ("source", "series float"),
            ("leftbars", "int"),
            ("rightbars", "int"),
        ],
        "series float"
    ),
    builtin_fn!(
        "ta.change",
        "Difference between current and previous value.",
        [("source", "series float"), ("length", "int"),],
        "series float"
    ),
    builtin_fn!(
        "ta.cum",
        "Cumulative sum of `source`.",
        [("source", "series float"),],
        "series float"
    ),
    builtin_fn!(
        "ta.tr",
        "True range.",
        [("handle_na", "bool"),],
        "series float"
    ),
    // ── math.* ───────────────────────────────────────────────────────────
    builtin_fn!(
        "math.abs",
        "Absolute value.",
        [("value", "series float")],
        "series float"
    ),
    builtin_fn!(
        "math.max",
        "Maximum of two or more values.",
        [("value1", "series float"), ("value2", "series float"),],
        "series float"
    ),
    builtin_fn!(
        "math.min",
        "Minimum of two or more values.",
        [("value1", "series float"), ("value2", "series float"),],
        "series float"
    ),
    builtin_fn!(
        "math.round",
        "Round to the nearest integer or specified precision.",
        [("value", "series float"), ("precision", "int"),],
        "series float"
    ),
    builtin_fn!(
        "math.floor",
        "Round down to the nearest integer.",
        [("value", "series float"),],
        "series int"
    ),
    builtin_fn!(
        "math.ceil",
        "Round up to the nearest integer.",
        [("value", "series float"),],
        "series int"
    ),
    builtin_fn!(
        "math.sqrt",
        "Square root.",
        [("value", "series float")],
        "series float"
    ),
    builtin_fn!(
        "math.pow",
        "Raise `base` to the power of `exponent`.",
        [("base", "series float"), ("exponent", "series float"),],
        "series float"
    ),
    builtin_fn!(
        "math.log",
        "Natural logarithm.",
        [("value", "series float")],
        "series float"
    ),
    builtin_fn!(
        "math.log10",
        "Base-10 logarithm.",
        [("value", "series float")],
        "series float"
    ),
    builtin_fn!(
        "math.avg",
        "Average of all arguments.",
        [("value1", "series float"), ("value2", "series float"),],
        "series float"
    ),
    builtin_fn!(
        "math.sum",
        "Sum of `source` over `length` bars.",
        [("source", "series float"), ("length", "int"),],
        "series float"
    ),
    // ── str.* ────────────────────────────────────────────────────────────
    builtin_fn!(
        "str.tostring",
        "Convert a value to its string representation.",
        [("value", "series float/int/bool"), ("format", "string"),],
        "string"
    ),
    builtin_fn!(
        "str.format",
        "Format a string with placeholders.",
        [("formatString", "string"), ("arg0", "any"),],
        "string"
    ),
    builtin_fn!(
        "str.length",
        "Return the length of a string.",
        [("s", "string")],
        "int"
    ),
    builtin_fn!(
        "str.contains",
        "True if `source` contains `str`.",
        [("source", "string"), ("str", "string"),],
        "bool"
    ),
    builtin_fn!(
        "str.replace",
        "Replace occurrences of `target` with `replacement`.",
        [
            ("source", "string"),
            ("target", "string"),
            ("replacement", "string"),
        ],
        "string"
    ),
    builtin_fn!(
        "str.split",
        "Split a string by the given separator.",
        [("s", "string"), ("separator", "string"),],
        "string[]"
    ),
    builtin_fn!(
        "str.upper",
        "Convert to upper case.",
        [("s", "string")],
        "string"
    ),
    builtin_fn!(
        "str.lower",
        "Convert to lower case.",
        [("s", "string")],
        "string"
    ),
    builtin_fn!(
        "str.startswith",
        "True if `source` starts with `str`.",
        [("source", "string"), ("str", "string"),],
        "bool"
    ),
    builtin_fn!(
        "str.endswith",
        "True if `source` ends with `str`.",
        [("source", "string"), ("str", "string"),],
        "bool"
    ),
    builtin_fn!(
        "str.substring",
        "Return a substring.",
        [("s", "string"), ("begin", "int"), ("end", "int"),],
        "string"
    ),
    // ── array.* ──────────────────────────────────────────────────────────
    builtin_fn!(
        "array.new_float",
        "Create a new float array.",
        [("size", "int"), ("initial_value", "float"),],
        "array<float>"
    ),
    builtin_fn!(
        "array.new_int",
        "Create a new int array.",
        [("size", "int"), ("initial_value", "int"),],
        "array<int>"
    ),
    builtin_fn!(
        "array.new_bool",
        "Create a new bool array.",
        [("size", "int"), ("initial_value", "bool"),],
        "array<bool>"
    ),
    builtin_fn!(
        "array.new_string",
        "Create a new string array.",
        [("size", "int"), ("initial_value", "string"),],
        "array<string>"
    ),
    builtin_fn!(
        "array.new_color",
        "Create a new color array.",
        [("size", "int"), ("initial_value", "color"),],
        "array<color>"
    ),
    builtin_fn!(
        "array.new_label",
        "Create a new label array.",
        [("size", "int"), ("initial_value", "label"),],
        "array<label>"
    ),
    builtin_fn!(
        "array.new_line",
        "Create a new line array.",
        [("size", "int"), ("initial_value", "line"),],
        "array<line>"
    ),
    builtin_fn!(
        "array.new_box",
        "Create a new box array.",
        [("size", "int"), ("initial_value", "box"),],
        "array<box>"
    ),
    builtin_fn!(
        "array.new_table",
        "Create a new table array.",
        [("size", "int"), ("initial_value", "table"),],
        "array<table>"
    ),
    builtin_fn!(
        "array.push",
        "Append an element to the end of the array.",
        [("id", "array"), ("value", "any"),],
        "void"
    ),
    builtin_fn!(
        "array.pop",
        "Remove and return the last element.",
        [("id", "array"),],
        "any"
    ),
    builtin_fn!(
        "array.get",
        "Return the element at `index`.",
        [("id", "array"), ("index", "int"),],
        "any"
    ),
    builtin_fn!(
        "array.set",
        "Set the value of the element at `index`.",
        [("id", "array"), ("index", "int"), ("value", "any"),],
        "void"
    ),
    builtin_fn!(
        "array.size",
        "Return the number of elements.",
        [("id", "array"),],
        "int"
    ),
    builtin_fn!(
        "array.from",
        "Create an array from a list of values.",
        [("val0", "any"),],
        "array"
    ),
    builtin_fn!(
        "array.sort",
        "Sort the array in place.",
        [("id", "array"), ("order", "sort_order"),],
        "void"
    ),
    // ── request.* ────────────────────────────────────────────────────────
    builtin_fn!(
        "request.security",
        "Request data from another symbol/timeframe.",
        [
            ("symbol", "string"),
            ("timeframe", "string"),
            ("expression", "any"),
            ("gaps", "barmerge_gaps"),
            ("lookahead", "barmerge_lookahead"),
            ("ignore_invalid_symbol", "bool"),
            ("currency", "string"),
        ],
        "any"
    ),
    builtin_fn!(
        "request.financial",
        "Request financial data for a symbol.",
        [
            ("symbol", "string"),
            ("financial_id", "string"),
            ("period", "string"),
            ("gaps", "barmerge_gaps"),
            ("ignore_invalid_symbol", "bool"),
            ("currency", "string"),
        ],
        "series float"
    ),
    builtin_fn!(
        "request.economic",
        "Request economic data.",
        [
            ("country_code", "string"),
            ("field", "string"),
            ("gaps", "barmerge_gaps"),
            ("ignore_invalid_symbol", "bool"),
        ],
        "series float"
    ),
    builtin_fn!(
        "request.dividends",
        "Request dividends data.",
        [
            ("ticker", "string"),
            ("field", "string"),
            ("gaps", "barmerge_gaps"),
            ("ignore_invalid_symbol", "bool"),
        ],
        "series float"
    ),
    builtin_fn!(
        "request.earnings",
        "Request earnings data.",
        [
            ("ticker", "string"),
            ("field", "string"),
            ("gaps", "barmerge_gaps"),
            ("ignore_invalid_symbol", "bool"),
        ],
        "series float"
    ),
    builtin_fn!(
        "request.splits",
        "Request stock split data.",
        [
            ("ticker", "string"),
            ("field", "string"),
            ("gaps", "barmerge_gaps"),
            ("ignore_invalid_symbol", "bool"),
        ],
        "series float"
    ),
    // ── strategy.* ───────────────────────────────────────────────────────
    builtin_fn!(
        "strategy.entry",
        "Enter a trade.",
        [
            ("id", "string"),
            ("direction", "strategy_direction"),
            ("qty", "float"),
            ("limit", "float"),
            ("stop", "float"),
            ("oca_name", "string"),
            ("oca_type", "string"),
            ("comment", "string"),
            ("alert_message", "string"),
        ],
        "void"
    ),
    builtin_fn!(
        "strategy.close",
        "Close a position.",
        [
            ("id", "string"),
            ("comment", "string"),
            ("qty", "float"),
            ("qty_percent", "float"),
            ("alert_message", "string"),
        ],
        "void"
    ),
    builtin_fn!(
        "strategy.exit",
        "Place an exit order.",
        [
            ("id", "string"),
            ("from_entry", "string"),
            ("qty", "float"),
            ("qty_percent", "float"),
            ("profit", "float"),
            ("limit", "float"),
            ("loss", "float"),
            ("stop", "float"),
            ("trail_price", "float"),
            ("trail_points", "float"),
            ("trail_offset", "float"),
            ("oca_name", "string"),
            ("comment", "string"),
            ("alert_message", "string"),
        ],
        "void"
    ),
    builtin_fn!(
        "strategy.order",
        "Place a generic order.",
        [
            ("id", "string"),
            ("direction", "strategy_direction"),
            ("qty", "float"),
            ("limit", "float"),
            ("stop", "float"),
            ("oca_name", "string"),
            ("oca_type", "string"),
            ("comment", "string"),
            ("alert_message", "string"),
        ],
        "void"
    ),
    builtin_fn!(
        "strategy.cancel",
        "Cancel an order by `id`.",
        [("id", "string"),],
        "void"
    ),
    // ── color.* ──────────────────────────────────────────────────────────
    builtin_fn!(
        "color.new",
        "Create a color with transparency.",
        [("color", "color"), ("transp", "series float"),],
        "color"
    ),
    builtin_fn!(
        "color.rgb",
        "Create a color from RGB values.",
        [
            ("red", "series int"),
            ("green", "series int"),
            ("blue", "series int"),
            ("transp", "series float"),
        ],
        "color"
    ),
    builtin_fn!(
        "color.r",
        "Get the red component.",
        [("color", "color")],
        "float"
    ),
    builtin_fn!(
        "color.g",
        "Get the green component.",
        [("color", "color")],
        "float"
    ),
    builtin_fn!(
        "color.b",
        "Get the blue component.",
        [("color", "color")],
        "float"
    ),
    // ── Alerts / logging ─────────────────────────────────────────────────
    builtin_fn!(
        "alert",
        "Trigger an alert.",
        [("message", "string"), ("freq", "alert_freq"),],
        "void"
    ),
    builtin_fn!(
        "alertcondition",
        "Create an alert condition.",
        [
            ("condition", "series bool"),
            ("title", "string"),
            ("message", "string"),
        ],
        "void"
    ),
    builtin_fn!(
        "log.info",
        "Log an informational message.",
        [("message", "string")],
        "void"
    ),
    builtin_fn!(
        "log.warning",
        "Log a warning message.",
        [("message", "string")],
        "void"
    ),
    builtin_fn!(
        "log.error",
        "Log an error message.",
        [("message", "string")],
        "void"
    ),
    // ── Drawing objects ──────────────────────────────────────────────────
    builtin_fn!(
        "label.new",
        "Create a new label on the chart.",
        [
            ("x", "series int"),
            ("y", "series float"),
            ("text", "string"),
            ("xloc", "string"),
            ("yloc", "string"),
            ("color", "color"),
            ("style", "label_style"),
            ("textcolor", "color"),
            ("size", "size"),
            ("textalign", "string"),
            ("tooltip", "string"),
        ],
        "label"
    ),
    builtin_fn!(
        "line.new",
        "Create a new line on the chart.",
        [
            ("x1", "series int"),
            ("y1", "series float"),
            ("x2", "series int"),
            ("y2", "series float"),
            ("xloc", "string"),
            ("extend", "string"),
            ("color", "color"),
            ("style", "line_style"),
            ("width", "int"),
        ],
        "line"
    ),
    builtin_fn!(
        "box.new",
        "Create a new box on the chart.",
        [
            ("left", "series int"),
            ("top", "series float"),
            ("right", "series int"),
            ("bottom", "series float"),
            ("border_color", "color"),
            ("border_width", "int"),
            ("border_style", "line_style"),
            ("extend", "string"),
            ("xloc", "string"),
            ("bgcolor", "color"),
            ("text", "string"),
            ("text_size", "size"),
            ("text_color", "color"),
            ("text_halign", "string"),
            ("text_valign", "string"),
            ("text_wrap", "string"),
        ],
        "box"
    ),
    builtin_fn!(
        "table.new",
        "Create a new table on the chart.",
        [
            ("position", "string"),
            ("columns", "int"),
            ("rows", "int"),
            ("bgcolor", "color"),
            ("frame_color", "color"),
            ("frame_width", "int"),
            ("border_color", "color"),
            ("border_width", "int"),
        ],
        "table"
    ),
    // ── Misc commonly used ───────────────────────────────────────────────
    builtin_fn!(
        "nz",
        "Replace `na` with zero or a specified value.",
        [("source", "series float/int"), ("replacement", "float/int"),],
        "series float/int"
    ),
    builtin_fn!(
        "fixnan",
        "Replace NaN values with the last non-NaN value.",
        [("source", "series float"),],
        "series float"
    ),
    builtin_fn!("na", "Test if a value is `na`.", [("value", "any")], "bool"),
    builtin_fn!(
        "input",
        "Add a generic input to the script settings.",
        [
            ("defval", "any"),
            ("title", "string"),
            ("tooltip", "string"),
            ("inline", "string"),
            ("group", "string"),
            ("confirm", "bool"),
        ],
        "any"
    ),
    builtin_fn!(
        "timestamp",
        "Return UNIX time for the given date.",
        [
            ("year", "int"),
            ("month", "int"),
            ("day", "int"),
            ("hour", "int"),
            ("minute", "int"),
            ("second", "int"),
        ],
        "int"
    ),
];

// ── Built-in variables / constants ────────────────────────────────────────────

macro_rules! builtin_var {
    ($name:expr, $doc:expr, $ty:expr) => {
        BuiltinVar {
            name: $name,
            doc: $doc,
            type_hint: $ty,
        }
    };
}

static BUILTIN_VARIABLES: &[BuiltinVar] = &[
    // Price / volume
    builtin_var!("open", "Opening price of the current bar.", "series float"),
    builtin_var!("high", "Highest price of the current bar.", "series float"),
    builtin_var!("low", "Lowest price of the current bar.", "series float"),
    builtin_var!("close", "Closing price of the current bar.", "series float"),
    builtin_var!("volume", "Volume of the current bar.", "series float"),
    builtin_var!("hl2", "(high + low) / 2.", "series float"),
    builtin_var!("hlc3", "(high + low + close) / 3.", "series float"),
    builtin_var!("ohlc4", "(open + high + low + close) / 4.", "series float"),
    builtin_var!("hlcc4", "(high + low + close + close) / 4.", "series float"),
    // Bar / time
    builtin_var!(
        "bar_index",
        "Index of the current bar (0-based).",
        "series int"
    ),
    builtin_var!(
        "time",
        "UNIX time of the current bar's open, in ms.",
        "series int"
    ),
    builtin_var!(
        "timenow",
        "Current real-world UNIX time in ms.",
        "series int"
    ),
    // Primitive constants
    builtin_var!("na", "Not available. Represents a missing value.", "na"),
    builtin_var!("true", "Boolean true literal.", "bool"),
    builtin_var!("false", "Boolean false literal.", "bool"),
    // syminfo.*
    builtin_var!(
        "syminfo.ticker",
        "The ticker of the current symbol without the exchange prefix.",
        "simple string"
    ),
    builtin_var!(
        "syminfo.tickerid",
        "The full ticker identifier including exchange prefix.",
        "simple string"
    ),
    builtin_var!(
        "syminfo.currency",
        "Currency of the current symbol.",
        "simple string"
    ),
    builtin_var!(
        "syminfo.description",
        "Description of the current symbol.",
        "simple string"
    ),
    builtin_var!(
        "syminfo.type",
        "Type of the current symbol (stock, crypto, etc.).",
        "simple string"
    ),
    builtin_var!(
        "syminfo.root",
        "Root of the current symbol.",
        "simple string"
    ),
    builtin_var!(
        "syminfo.prefix",
        "Exchange prefix for the current symbol.",
        "simple string"
    ),
    builtin_var!(
        "syminfo.timezone",
        "Timezone of the exchange.",
        "simple string"
    ),
    builtin_var!(
        "syminfo.session",
        "Session type of the symbol.",
        "simple string"
    ),
    builtin_var!(
        "syminfo.basecurrency",
        "Base currency for currency pairs.",
        "simple string"
    ),
    builtin_var!(
        "syminfo.mintick",
        "Minimum tick value for the symbol.",
        "simple float"
    ),
    builtin_var!(
        "syminfo.pointvalue",
        "Point value for the symbol.",
        "simple float"
    ),
    builtin_var!("syminfo.volumetype", "Volume type.", "simple string"),
    // timeframe.*
    builtin_var!(
        "timeframe.period",
        "The timeframe string of the chart.",
        "simple string"
    ),
    builtin_var!(
        "timeframe.multiplier",
        "The multiplier of the chart timeframe.",
        "simple int"
    ),
    builtin_var!(
        "timeframe.isintraday",
        "True if the timeframe is intraday.",
        "simple bool"
    ),
    builtin_var!(
        "timeframe.isdaily",
        "True if the timeframe is daily.",
        "simple bool"
    ),
    builtin_var!(
        "timeframe.isweekly",
        "True if the timeframe is weekly.",
        "simple bool"
    ),
    builtin_var!(
        "timeframe.ismonthly",
        "True if the timeframe is monthly.",
        "simple bool"
    ),
    builtin_var!(
        "timeframe.isdwm",
        "True if daily, weekly or monthly.",
        "simple bool"
    ),
    builtin_var!(
        "timeframe.isseconds",
        "True if the timeframe is in seconds.",
        "simple bool"
    ),
    builtin_var!(
        "timeframe.isminutes",
        "True if the timeframe is in minutes.",
        "simple bool"
    ),
    // Bar state
    builtin_var!("barstate.isfirst", "True on the first bar.", "series bool"),
    builtin_var!("barstate.islast", "True on the last bar.", "series bool"),
    builtin_var!(
        "barstate.ishistory",
        "True on historical bars.",
        "series bool"
    ),
    builtin_var!(
        "barstate.isrealtime",
        "True on real-time bars.",
        "series bool"
    ),
    builtin_var!(
        "barstate.isnew",
        "True if the script is executing on a new bar.",
        "series bool"
    ),
    builtin_var!(
        "barstate.isconfirmed",
        "True if the bar is confirmed (closed).",
        "series bool"
    ),
    builtin_var!(
        "barstate.islastconfirmedhistory",
        "True on the last confirmed historical bar.",
        "series bool"
    ),
    // strategy.*
    builtin_var!(
        "strategy.position_size",
        "Current position size.",
        "series float"
    ),
    builtin_var!("strategy.equity", "Current equity.", "series float"),
    builtin_var!(
        "strategy.openprofit",
        "Current open profit.",
        "series float"
    ),
    builtin_var!("strategy.netprofit", "Net profit.", "series float"),
    builtin_var!("strategy.grossprofit", "Gross profit.", "series float"),
    builtin_var!("strategy.grossloss", "Gross loss.", "series float"),
    builtin_var!(
        "strategy.closedtrades",
        "Number of closed trades.",
        "series int"
    ),
    builtin_var!(
        "strategy.wintrades",
        "Number of winning trades.",
        "series int"
    ),
    builtin_var!(
        "strategy.losstrades",
        "Number of losing trades.",
        "series int"
    ),
    builtin_var!(
        "strategy.long",
        "Direction constant: long.",
        "strategy_direction"
    ),
    builtin_var!(
        "strategy.short",
        "Direction constant: short.",
        "strategy_direction"
    ),
    // ta.*
    builtin_var!("ta.accdist", "Accumulation/Distribution.", "series float"),
    // color constants
    builtin_var!("color.red", "Built-in color constant: red.", "color"),
    builtin_var!("color.green", "Built-in color constant: green.", "color"),
    builtin_var!("color.blue", "Built-in color constant: blue.", "color"),
    builtin_var!("color.white", "Built-in color constant: white.", "color"),
    builtin_var!("color.black", "Built-in color constant: black.", "color"),
    builtin_var!("color.yellow", "Built-in color constant: yellow.", "color"),
    builtin_var!("color.orange", "Built-in color constant: orange.", "color"),
    builtin_var!("color.purple", "Built-in color constant: purple.", "color"),
    builtin_var!("color.aqua", "Built-in color constant: aqua.", "color"),
    builtin_var!("color.silver", "Built-in color constant: silver.", "color"),
    builtin_var!("color.gray", "Built-in color constant: gray.", "color"),
    builtin_var!("color.lime", "Built-in color constant: lime.", "color"),
    builtin_var!("color.maroon", "Built-in color constant: maroon.", "color"),
    builtin_var!("color.navy", "Built-in color constant: navy.", "color"),
    builtin_var!("color.olive", "Built-in color constant: olive.", "color"),
    builtin_var!(
        "color.fuchsia",
        "Built-in color constant: fuchsia.",
        "color"
    ),
    builtin_var!("color.teal", "Built-in color constant: teal.", "color"),
    // display.*
    builtin_var!("display.all", "Display flag: all.", "int"),
    builtin_var!("display.none", "Display flag: none.", "int"),
    builtin_var!("display.pane", "Display flag: pane.", "int"),
    builtin_var!("display.data_window", "Display flag: data window.", "int"),
    builtin_var!("display.status_line", "Display flag: status line.", "int"),
    builtin_var!("display.price_scale", "Display flag: price scale.", "int"),
    // last_bar_*
    builtin_var!("last_bar_index", "Index of the last bar.", "series int"),
    builtin_var!(
        "last_bar_time",
        "UNIX time of the last bar open.",
        "series int"
    ),
];

// ── Deprecated functions ──────────────────────────────────────────────────────

static DEPRECATED_FUNCTIONS: &[DeprecatedFunc] = &[
    DeprecatedFunc {
        old_name: "study",
        new_name: "indicator",
        message: "`study()` is deprecated in Pine Script v6. Use `indicator()` instead.",
    },
    DeprecatedFunc {
        old_name: "security",
        new_name: "request.security",
        message: "`security()` is deprecated in Pine Script v6. Use `request.security()` instead.",
    },
    DeprecatedFunc {
        old_name: "tickerid",
        new_name: "syminfo.tickerid",
        message: "`tickerid` is deprecated in Pine Script v6. Use `syminfo.tickerid` instead.",
    },
    DeprecatedFunc {
        old_name: "tostring",
        new_name: "str.tostring",
        message: "`tostring()` is deprecated in Pine Script v6. Use `str.tostring()` instead.",
    },
    DeprecatedFunc {
        old_name: "color",
        new_name: "color.new",
        message: "`color()` with transparency is deprecated. Use `color.new()` or `color.rgb()`.",
    },
    DeprecatedFunc {
        old_name: "input.resolution",
        new_name: "input.timeframe",
        message: "`input.resolution()` is deprecated. Use `input.timeframe()` instead.",
    },
    DeprecatedFunc {
        old_name: "iff",
        new_name: "ternary operator",
        message: "`iff()` is deprecated. Use the ternary operator `cond ? a : b` instead.",
    },
    DeprecatedFunc {
        old_name: "rsi",
        new_name: "ta.rsi",
        message: "`rsi()` is deprecated. Use `ta.rsi()` instead.",
    },
    DeprecatedFunc {
        old_name: "sma",
        new_name: "ta.sma",
        message: "`sma()` is deprecated. Use `ta.sma()` instead.",
    },
    DeprecatedFunc {
        old_name: "ema",
        new_name: "ta.ema",
        message: "`ema()` is deprecated. Use `ta.ema()` instead.",
    },
    DeprecatedFunc {
        old_name: "macd",
        new_name: "ta.macd",
        message: "`macd()` is deprecated. Use `ta.macd()` instead.",
    },
    DeprecatedFunc {
        old_name: "atr",
        new_name: "ta.atr",
        message: "`atr()` is deprecated. Use `ta.atr()` instead.",
    },
    DeprecatedFunc {
        old_name: "crossover",
        new_name: "ta.crossover",
        message: "`crossover()` is deprecated. Use `ta.crossover()` instead.",
    },
    DeprecatedFunc {
        old_name: "crossunder",
        new_name: "ta.crossunder",
        message: "`crossunder()` is deprecated. Use `ta.crossunder()` instead.",
    },
    DeprecatedFunc {
        old_name: "highest",
        new_name: "ta.highest",
        message: "`highest()` is deprecated. Use `ta.highest()` instead.",
    },
    DeprecatedFunc {
        old_name: "lowest",
        new_name: "ta.lowest",
        message: "`lowest()` is deprecated. Use `ta.lowest()` instead.",
    },
    DeprecatedFunc {
        old_name: "stoch",
        new_name: "ta.stoch",
        message: "`stoch()` is deprecated. Use `ta.stoch()` instead.",
    },
    DeprecatedFunc {
        old_name: "tr",
        new_name: "ta.tr",
        message: "`tr()` is deprecated. Use `ta.tr()` instead.",
    },
    DeprecatedFunc {
        old_name: "vwap",
        new_name: "ta.vwap",
        message: "`vwap()` is deprecated. Use `ta.vwap()` instead.",
    },
    DeprecatedFunc {
        old_name: "cum",
        new_name: "ta.cum",
        message: "`cum()` is deprecated. Use `ta.cum()` instead.",
    },
    DeprecatedFunc {
        old_name: "change",
        new_name: "ta.change",
        message: "`change()` is deprecated. Use `ta.change()` instead.",
    },
    DeprecatedFunc {
        old_name: "abs",
        new_name: "math.abs",
        message: "`abs()` is deprecated. Use `math.abs()` instead.",
    },
    DeprecatedFunc {
        old_name: "round",
        new_name: "math.round",
        message: "`round()` is deprecated. Use `math.round()` instead.",
    },
    DeprecatedFunc {
        old_name: "floor",
        new_name: "math.floor",
        message: "`floor()` is deprecated. Use `math.floor()` instead.",
    },
    DeprecatedFunc {
        old_name: "ceil",
        new_name: "math.ceil",
        message: "`ceil()` is deprecated. Use `math.ceil()` instead.",
    },
    DeprecatedFunc {
        old_name: "sqrt",
        new_name: "math.sqrt",
        message: "`sqrt()` is deprecated. Use `math.sqrt()` instead.",
    },
    DeprecatedFunc {
        old_name: "pow",
        new_name: "math.pow",
        message: "`pow()` is deprecated. Use `math.pow()` instead.",
    },
    DeprecatedFunc {
        old_name: "log",
        new_name: "math.log",
        message: "`log()` is deprecated. Use `math.log()` instead.",
    },
    DeprecatedFunc {
        old_name: "log10",
        new_name: "math.log10",
        message: "`log10()` is deprecated. Use `math.log10()` instead.",
    },
    DeprecatedFunc {
        old_name: "avg",
        new_name: "math.avg",
        message: "`avg()` is deprecated. Use `math.avg()` instead.",
    },
    DeprecatedFunc {
        old_name: "sum",
        new_name: "math.sum",
        message: "`sum()` is deprecated. Use `math.sum()` instead.",
    },
    DeprecatedFunc {
        old_name: "max",
        new_name: "math.max",
        message: "`max()` is deprecated. Use `math.max()` instead.",
    },
    DeprecatedFunc {
        old_name: "min",
        new_name: "math.min",
        message: "`min()` is deprecated. Use `math.min()` instead.",
    },
];

// ── Pine Script v6 keywords ──────────────────────────────────────────────────

static KEYWORDS: &[&str] = &[
    "if",
    "else",
    "for",
    "to",
    "by",
    "in",
    "while",
    "switch",
    "var",
    "varip",
    "true",
    "false",
    "na",
    "and",
    "or",
    "not",
    "import",
    "export",
    "as",
    "type",
    "enum",
    "method",
    "series",
    "simple",
    "input",
    "const",
    "int",
    "float",
    "bool",
    "string",
    "color",
    "label",
    "line",
    "box",
    "table",
    "linefill",
    "polyline",
    "array",
    "matrix",
    "map",
    "return",
    "break",
    "continue",
    "indicator",
    "strategy",
    "library",
];

// ── Public lookup API ─────────────────────────────────────────────────────────

/// Look up a built-in function by its full name (e.g. `"ta.sma"` or `"plot"`).
pub fn lookup_function(name: &str) -> Option<&'static BuiltinFunc> {
    BUILTIN_FUNCTIONS.iter().find(|f| f.name == name)
}

/// Look up a built-in variable / constant by name (e.g. `"close"`,
/// `"syminfo.ticker"`).
pub fn lookup_variable(name: &str) -> Option<&'static BuiltinVar> {
    BUILTIN_VARIABLES.iter().find(|v| v.name == name)
}

/// Look up a deprecated function by its old name.
pub fn lookup_deprecated(name: &str) -> Option<&'static DeprecatedFunc> {
    DEPRECATED_FUNCTIONS.iter().find(|d| d.old_name == name)
}

/// Return every registered built-in function.
pub fn all_functions() -> &'static [BuiltinFunc] {
    BUILTIN_FUNCTIONS
}

/// Return every registered built-in variable / constant.
pub fn all_variables() -> &'static [BuiltinVar] {
    BUILTIN_VARIABLES
}

/// Return every deprecated function entry.
pub fn all_deprecated() -> &'static [DeprecatedFunc] {
    DEPRECATED_FUNCTIONS
}

/// Return Pine Script v6 keywords.
pub fn all_keywords() -> Vec<&'static str> {
    KEYWORDS.to_vec()
}

/// Returns `true` when `name` is a known built-in identifier — either a
/// function, a variable/constant, or a keyword.
pub fn is_known_builtin(name: &str) -> bool {
    lookup_function(name).is_some() || lookup_variable(name).is_some() || KEYWORDS.contains(&name)
}

/// Returns `true` when `name` is the prefix of a known namespace
/// (e.g. `"ta"`, `"math"`, `"str"`, `"array"`, `"request"`, etc.).
pub fn is_namespace_prefix(name: &str) -> bool {
    static NAMESPACES: &[&str] = &[
        "ta",
        "math",
        "str",
        "array",
        "matrix",
        "map",
        "request",
        "strategy",
        "color",
        "input",
        "log",
        "label",
        "line",
        "box",
        "table",
        "linefill",
        "polyline",
        "syminfo",
        "timeframe",
        "barstate",
        "display",
        "chart",
        "session",
        "dayofweek",
        "ticker",
        "format",
        "shape",
        "location",
        "size",
        "barmerge",
        "xloc",
        "yloc",
        "text",
        "font",
        "adjustment",
        "currency",
        "earnings",
        "dividends",
        "splits",
        "extend",
        "scale",
        "alert",
        "plot",
        "hline",
        "position",
    ];
    NAMESPACES.contains(&name)
}
