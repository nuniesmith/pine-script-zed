// ── Span ──────────────────────────────────────────────────────────────────────

/// Byte-offset span into the *original* source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// A node `T` together with its source span.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // Primitives
    Int,
    Float,
    Bool,
    String,
    Color,
    Void,

    // Drawing types
    Label,
    Line,
    Box,
    Table,
    Linefill,
    Polyline,

    // Qualifiers wrapping an inner type
    Series(Box<Type>),
    Simple(Box<Type>),
    Input(Box<Type>),
    Const(Box<Type>),

    // Generic containers
    Array(Box<Type>),
    Matrix(Box<Type>),
    Map(Box<Type>, Box<Type>),

    // User-defined or unresolved type name
    Named(String),
}

// ── Expressions ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    StringLit(String),
    ColorLit(String), // e.g. #ff0000, #ff0000ff
    Na,

    // Identifiers
    Ident(String),

    // Binary operation
    BinOp {
        op: BinOp,
        lhs: Box<Spanned<Expr>>,
        rhs: Box<Spanned<Expr>>,
    },

    // Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<Spanned<Expr>>,
    },

    // Function / method call
    Call {
        func: Box<Spanned<Expr>>,
        args: Vec<Spanned<Expr>>,
        named: Vec<(String, Spanned<Expr>)>,
    },

    // Subscript: expr[index]
    Index {
        object: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
    },

    // Field access: expr.field
    Field {
        object: Box<Spanned<Expr>>,
        field: String,
    },

    // Ternary: cond ? then : else
    Ternary {
        cond: Box<Spanned<Expr>>,
        then_expr: Box<Spanned<Expr>>,
        else_expr: Box<Spanned<Expr>>,
    },

    // Tuple-like grouping: [a, b, c]
    Tuple(Vec<Spanned<Expr>>),

    // Type cast: type(expr)
    Cast {
        target_type: Type,
        expr: Box<Spanned<Expr>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not, // `not` keyword in Pine Script v6
}

// ── Variable declaration ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    /// Plain `x = expr`
    Decl,
    /// `var x = expr`
    Var,
    /// `varip x = expr`
    Varip,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub kind: VarKind,
    pub name: String,
    pub type_ann: Option<Type>,
    pub value: Spanned<Expr>,
}

// ── Function / method parameters ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<Type>,
    pub default: Option<Spanned<Expr>>,
}

// ── Function definition ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: Option<Type>,
    pub body: Vec<Spanned<Stmt>>,
}

// ── Method definition ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MethodDef {
    pub receiver_type: String,
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: Option<Type>,
    pub body: Vec<Spanned<Stmt>>,
}

// ── User-defined type ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TypeField {
    pub name: String,
    pub type_ann: Type,
    pub default: Option<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub export: bool,
    pub name: String,
    pub fields: Vec<TypeField>,
}

// ── Enum definition (Pine v6) ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub export: bool,
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

// ── Import ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ImportDef {
    /// e.g. "TradingView/ta/5" or "user/library/1"
    pub path: String,
    /// Optional `as` alias
    pub alias: Option<String>,
}

// ── Switch arm ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SwitchArm {
    Case(Spanned<Expr>, Vec<Spanned<Stmt>>),
    Default(Vec<Spanned<Stmt>>),
}

// ── Statements ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `[var|varip] [type] name = expr`
    VarDecl(VarDecl),

    /// `target := value`
    Reassign {
        target: Spanned<Expr>,
        value: Spanned<Expr>,
    },

    /// `name(params) => body`
    FuncDef(FuncDef),

    /// `method name(self_type, params) => body`
    MethodDef(MethodDef),

    /// `type Name \n  field1: type \n ...`
    TypeDef(TypeDef),

    /// `enum Name \n  variant1 \n ...`
    EnumDef(EnumDef),

    /// `import "path" as alias`
    Import(ImportDef),

    /// `export name`
    Export(String),

    /// `if cond \n ... \n else if cond \n ... \n else \n ...`
    If {
        cond: Spanned<Expr>,
        then_body: Vec<Spanned<Stmt>>,
        else_ifs: Vec<(Spanned<Expr>, Vec<Spanned<Stmt>>)>,
        else_body: Option<Vec<Spanned<Stmt>>>,
    },

    /// `switch [expr] \n  case1 => body \n ...`
    Switch {
        expr: Option<Spanned<Expr>>,
        arms: Vec<SwitchArm>,
    },

    /// `for var = from to to [by step] \n body`
    For {
        var: String,
        from: Spanned<Expr>,
        to: Spanned<Expr>,
        step: Option<Spanned<Expr>>,
        body: Vec<Spanned<Stmt>>,
    },

    /// `for [key,] value in iterable \n body`
    ForIn {
        key_var: Option<String>,
        val_var: String,
        iterable: Spanned<Expr>,
        body: Vec<Spanned<Stmt>>,
    },

    /// `while cond \n body`
    While {
        cond: Spanned<Expr>,
        body: Vec<Spanned<Stmt>>,
    },

    Return(Option<Spanned<Expr>>),
    Break,
    Continue,

    /// Bare expression used as a statement.
    Expr(Spanned<Expr>),
}

// ── Script-level declarations ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptKind {
    Indicator,
    Strategy,
    Library,
}

#[derive(Debug, Clone)]
pub struct Script {
    /// The `//@version=N` value, if present.
    pub version: Option<u8>,

    /// The script declaration kind + its call expression.
    pub kind: Option<(ScriptKind, Spanned<Expr>)>,

    /// All top-level statements (including imports at the top).
    pub stmts: Vec<Spanned<Stmt>>,
}

// ── Parse error ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

/// Result bundle from the parser.
#[derive(Debug)]
pub struct ParseResult {
    pub script: Option<Script>,
    pub errors: Vec<ParseError>,
}

// ── Lint diagnostic ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    pub message: String,
    pub span: Span,
    pub severity: LintSeverity,
}
