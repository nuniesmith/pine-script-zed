use std::collections::HashMap;

use crate::ast::*;
use crate::builtins;

// ── Public entry point ────────────────────────────────────────────────────────

/// Lint a parsed Pine Script AST and return all diagnostics.
pub fn lint(script: &Script, _source: &str) -> Vec<LintDiagnostic> {
    let mut linter = Linter::new();
    linter.lint_script(script);
    linter.finish()
}

// ── Internal types ────────────────────────────────────────────────────────────

struct VarInfo {
    span: Span,
    used: bool,
    type_ann: Option<Type>,
}

struct Scope {
    vars: HashMap<String, VarInfo>,
}

impl Scope {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

struct Linter {
    diags: Vec<LintDiagnostic>,
    scopes: Vec<Scope>,
    in_loop: usize, // nesting depth – 0 means outside any loop
}

impl Linter {
    fn new() -> Self {
        Self {
            diags: Vec::new(),
            scopes: vec![Scope::new()], // global scope
            in_loop: 0,
        }
    }

    // ── helpers ───────────────────────────────────────────────────────────

    fn emit(&mut self, message: impl Into<String>, span: Span, severity: LintSeverity) {
        self.diags.push(LintDiagnostic {
            message: message.into(),
            span,
            severity,
        });
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            // Rule: unused_variables
            for (name, info) in &scope.vars {
                if !info.used && !name.starts_with('_') {
                    self.diags.push(LintDiagnostic {
                        message: format!("Variable `{}` is declared but never used", name),
                        span: info.span.clone(),
                        severity: LintSeverity::Hint,
                    });
                }
            }
        }
    }

    /// Declare a variable in the current (innermost) scope.
    fn declare_var(&mut self, name: &str, span: &Span, type_ann: Option<Type>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.vars.insert(
                name.to_string(),
                VarInfo {
                    span: span.clone(),
                    used: false,
                    type_ann,
                },
            );
        }
    }

    /// Mark a variable as used, searching from innermost scope outward.
    /// Returns `true` if the variable was found.
    fn mark_used(&mut self, name: &str) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.vars.get_mut(name) {
                info.used = true;
                return true;
            }
        }
        false
    }

    /// Check whether a variable has been declared in any visible scope.
    fn is_declared(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.vars.contains_key(name))
    }

    /// Consume the linter and return the collected diagnostics.
    fn finish(mut self) -> Vec<LintDiagnostic> {
        // Pop remaining scopes (collects unused-variable hints for the global
        // scope).  We iterate in reverse so inner scopes are popped first.
        while self.scopes.len() > 1 {
            self.pop_scope();
        }
        // Pop global scope last
        self.pop_scope();
        self.diags
    }

    // ── top-level ─────────────────────────────────────────────────────────

    fn lint_script(&mut self, script: &Script) {
        // Rule: version_check
        match script.version {
            None => {
                self.emit(
                    "Missing `//@version=6` directive",
                    Span::new(0, 0),
                    LintSeverity::Warning,
                );
            }
            Some(v) if v != 6 => {
                self.emit(
                    format!("This linter targets Pine Script v6; you are using v{}", v),
                    Span::new(0, 0),
                    LintSeverity::Info,
                );
            }
            _ => {}
        }

        // Rule: script_declaration
        if script.kind.is_none() {
            self.emit(
                "Script is missing an `indicator()`, `strategy()`, or `library()` declaration",
                Span::new(0, 0),
                LintSeverity::Warning,
            );
        }

        for stmt in &script.stmts {
            self.walk_stmt(stmt);
        }
    }

    // ── statement walker ──────────────────────────────────────────────────

    fn walk_stmt(&mut self, spanned: &Spanned<Stmt>) {
        match &spanned.node {
            Stmt::VarDecl(decl) => {
                // Walk the initialiser *before* declaring the variable so that
                // self-references are flagged.
                self.walk_expr(&decl.value);

                // Rule: type_mismatch_basic
                if let Some(ref ty) = decl.type_ann {
                    self.check_type_mismatch(ty, &decl.value, &spanned.span);
                }

                self.declare_var(&decl.name, &spanned.span, decl.type_ann.clone());
            }

            Stmt::Reassign { target, value } => {
                // Rule: reassign_without_decl
                if let Expr::Ident(name) = &target.node {
                    if !self.is_declared(name) && !builtins::is_known_builtin(name) {
                        self.emit(
                            format!("Reassignment `:=` to undeclared variable `{}`", name),
                            target.span.clone(),
                            LintSeverity::Error,
                        );
                    } else {
                        self.mark_used(name);
                    }
                }
                self.walk_expr(target);
                self.walk_expr(value);
            }

            Stmt::FuncDef(func) => {
                // The function name is visible in the outer scope.
                self.declare_var(&func.name, &spanned.span, func.ret_type.clone());
                // Mark immediately as "used" – function declarations should not
                // trigger unused-variable warnings by default.
                self.mark_used(&func.name);

                self.push_scope();
                for param in &func.params {
                    self.declare_var(&param.name, &spanned.span, param.type_ann.clone());
                    // Params with defaults – walk the default expression.
                    if let Some(ref def) = param.default {
                        self.walk_expr(def);
                    }
                }
                for s in &func.body {
                    self.walk_stmt(s);
                }
                self.pop_scope();
            }

            Stmt::MethodDef(method) => {
                self.push_scope();
                // Receiver is implicitly declared as the first parameter.
                self.declare_var(&method.receiver_type, &spanned.span, None);
                for param in &method.params {
                    self.declare_var(&param.name, &spanned.span, param.type_ann.clone());
                    if let Some(ref def) = param.default {
                        self.walk_expr(def);
                    }
                }
                for s in &method.body {
                    self.walk_stmt(s);
                }
                self.pop_scope();
            }

            Stmt::TypeDef(td) => {
                // Register the type name in the current scope.
                self.declare_var(&td.name, &spanned.span, Some(Type::Named(td.name.clone())));
                self.mark_used(&td.name);

                for field in &td.fields {
                    if let Some(ref def) = field.default {
                        self.walk_expr(def);
                    }
                }
            }

            Stmt::EnumDef(ed) => {
                self.declare_var(&ed.name, &spanned.span, Some(Type::Named(ed.name.clone())));
                self.mark_used(&ed.name);

                for variant in &ed.variants {
                    if let Some(ref val) = variant.value {
                        self.walk_expr(val);
                    }
                }
            }

            Stmt::Import(imp) => {
                // If there's an alias, declare it.
                if let Some(ref alias) = imp.alias {
                    self.declare_var(alias, &spanned.span, None);
                    self.mark_used(alias);
                }
            }

            Stmt::Export(_) => { /* nothing to lint */ }

            Stmt::If {
                cond,
                then_body,
                else_ifs,
                else_body,
            } => {
                self.walk_expr(cond);

                self.push_scope();
                for s in then_body {
                    self.walk_stmt(s);
                }
                self.pop_scope();

                for (ei_cond, ei_body) in else_ifs {
                    self.walk_expr(ei_cond);
                    self.push_scope();
                    for s in ei_body {
                        self.walk_stmt(s);
                    }
                    self.pop_scope();
                }

                if let Some(eb) = else_body {
                    self.push_scope();
                    for s in eb {
                        self.walk_stmt(s);
                    }
                    self.pop_scope();
                }
            }

            Stmt::Switch { expr, arms } => {
                if let Some(e) = expr {
                    self.walk_expr(e);
                }
                for arm in arms {
                    match arm {
                        SwitchArm::Case(e, body) => {
                            self.walk_expr(e);
                            self.push_scope();
                            for s in body.iter() {
                                self.walk_stmt(s);
                            }
                            self.pop_scope();
                        }
                        SwitchArm::Default(body) => {
                            self.push_scope();
                            for s in body.iter() {
                                self.walk_stmt(s);
                            }
                            self.pop_scope();
                        }
                    }
                }
            }

            Stmt::For {
                var,
                from,
                to,
                step,
                body,
            } => {
                self.walk_expr(from);
                self.walk_expr(to);
                if let Some(s) = step {
                    self.walk_expr(s);
                }

                self.push_scope();
                self.declare_var(var, &spanned.span, Some(Type::Int));
                self.in_loop += 1;
                for s in body.iter() {
                    self.walk_stmt(s);
                }
                self.in_loop -= 1;
                self.pop_scope();
            }

            Stmt::ForIn {
                key_var,
                val_var,
                iterable,
                body,
            } => {
                self.walk_expr(iterable);

                self.push_scope();
                if let Some(k) = key_var {
                    self.declare_var(k, &spanned.span, Some(Type::Int));
                }
                self.declare_var(val_var, &spanned.span, None);
                self.in_loop += 1;
                for s in body.iter() {
                    self.walk_stmt(s);
                }
                self.in_loop -= 1;
                self.pop_scope();
            }

            Stmt::While { cond, body } => {
                self.walk_expr(cond);

                self.push_scope();
                self.in_loop += 1;
                for s in body.iter() {
                    self.walk_stmt(s);
                }
                self.in_loop -= 1;
                self.pop_scope();
            }

            Stmt::Return(opt_expr) => {
                if let Some(e) = opt_expr {
                    self.walk_expr(e);
                }
            }

            // Rule: break_continue_outside_loop
            Stmt::Break => {
                if self.in_loop == 0 {
                    self.emit(
                        "`break` used outside of a loop",
                        spanned.span.clone(),
                        LintSeverity::Error,
                    );
                }
            }
            Stmt::Continue => {
                if self.in_loop == 0 {
                    self.emit(
                        "`continue` used outside of a loop",
                        spanned.span.clone(),
                        LintSeverity::Error,
                    );
                }
            }

            Stmt::Expr(e) => {
                self.walk_expr(e);
            }
        }
    }

    // ── expression walker ─────────────────────────────────────────────────

    fn walk_expr(&mut self, spanned: &Spanned<Expr>) {
        match &spanned.node {
            Expr::IntLit(_)
            | Expr::FloatLit(_)
            | Expr::BoolLit(_)
            | Expr::StringLit(_)
            | Expr::ColorLit(_)
            | Expr::Na => {}

            Expr::Ident(name) => {
                // Try to mark as used in local scopes first.
                if !self.mark_used(name) {
                    // Not declared locally — is it a known built-in?
                    if !builtins::is_known_builtin(name) && !builtins::is_namespace_prefix(name) {
                        self.emit(
                            format!("Possibly undeclared variable `{}`", name),
                            spanned.span.clone(),
                            LintSeverity::Warning,
                        );
                    }
                }
            }

            Expr::BinOp { lhs, rhs, .. } => {
                self.walk_expr(lhs);
                self.walk_expr(rhs);
            }

            Expr::UnaryOp { operand, .. } => {
                self.walk_expr(operand);
            }

            Expr::Call { func, args, named } => {
                // Extract the full dotted function name for built-in checks.
                let func_name = self.extract_callable_name(func);

                // Rule: deprecated_function
                if let Some(ref name) = func_name {
                    if let Some(dep) = builtins::lookup_deprecated(name) {
                        self.emit(
                            dep.message.to_string(),
                            func.span.clone(),
                            LintSeverity::Warning,
                        );
                    }
                }

                self.walk_expr(func);

                for arg in args {
                    self.walk_expr(arg);
                }
                for (_name, val) in named {
                    self.walk_expr(val);
                }
            }

            Expr::Index { object, index } => {
                self.walk_expr(object);
                self.walk_expr(index);
            }

            Expr::Field { object, .. } => {
                self.walk_expr(object);
            }

            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                self.walk_expr(cond);
                self.walk_expr(then_expr);
                self.walk_expr(else_expr);
            }

            Expr::Tuple(elems) => {
                for e in elems {
                    self.walk_expr(e);
                }
            }

            Expr::Cast { expr, .. } => {
                self.walk_expr(expr);
            }
        }
    }

    // ── helper: extract a dotted name from a Call's `func` expression ─────

    /// Try to reconstruct a dotted name like `"ta.sma"` from the func
    /// expression of a Call node.  Returns `None` for complex expressions.
    fn extract_callable_name(&self, expr: &Spanned<Expr>) -> Option<String> {
        match &expr.node {
            Expr::Ident(name) => Some(name.clone()),
            Expr::Field { object, field } => {
                let prefix = self.extract_callable_name(object)?;
                Some(format!("{}.{}", prefix, field))
            }
            _ => None,
        }
    }

    // ── Rule: type_mismatch_basic ─────────────────────────────────────────

    /// Very basic type-mismatch check: if a variable declaration has a type
    /// annotation and its initialiser is a *literal* of a clearly incompatible
    /// type, emit an error.
    fn check_type_mismatch(&mut self, declared: &Type, value: &Spanned<Expr>, decl_span: &Span) {
        let base_type = Self::unwrap_qualifier(declared);

        let literal_type = match &value.node {
            Expr::IntLit(_) => Some(Type::Int),
            Expr::FloatLit(_) => Some(Type::Float),
            Expr::BoolLit(_) => Some(Type::Bool),
            Expr::StringLit(_) => Some(Type::String),
            Expr::ColorLit(_) => Some(Type::Color),
            _ => None,
        };

        let lit_type = match literal_type {
            Some(t) => t,
            None => return, // not a literal – skip the check
        };

        // int ← float is not a mismatch in Pine (implicit conversion), but
        // anything else that differs is suspicious.
        if !Self::types_compatible(base_type, &lit_type) {
            self.emit(
                format!(
                    "Type mismatch: expected `{}` but found `{}` literal",
                    Self::type_label(base_type),
                    Self::type_label(&lit_type),
                ),
                decl_span.clone(),
                LintSeverity::Error,
            );
        }
    }

    /// Strip qualifiers (series, simple, input, const) to get the base type.
    fn unwrap_qualifier(ty: &Type) -> &Type {
        match ty {
            Type::Series(inner) | Type::Simple(inner) | Type::Input(inner) | Type::Const(inner) => {
                Self::unwrap_qualifier(inner)
            }
            other => other,
        }
    }

    /// Very lenient compatibility check between two *base* types.
    fn types_compatible(declared: &Type, literal: &Type) -> bool {
        match (declared, literal) {
            // Exact match
            (a, b) if a == b => true,
            // int ↔ float are compatible in Pine
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
            // Named types — we can't tell, so assume compatible
            (Type::Named(_), _) | (_, Type::Named(_)) => true,
            // Void is compatible with everything (assignment to void is rare
            // but not our job to catch here)
            (Type::Void, _) | (_, Type::Void) => true,
            _ => false,
        }
    }

    fn type_label(ty: &Type) -> &'static str {
        match ty {
            Type::Int => "int",
            Type::Float => "float",
            Type::Bool => "bool",
            Type::String => "string",
            Type::Color => "color",
            Type::Void => "void",
            Type::Label => "label",
            Type::Line => "line",
            Type::Box => "box",
            Type::Table => "table",
            Type::Linefill => "linefill",
            Type::Polyline => "polyline",
            Type::Series(_) => "series",
            Type::Simple(_) => "simple",
            Type::Input(_) => "input",
            Type::Const(_) => "const",
            Type::Array(_) => "array",
            Type::Matrix(_) => "matrix",
            Type::Map(_, _) => "map",
            Type::Named(_) => "named",
        }
    }
}
