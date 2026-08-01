//! Render `Term`s back to surface syntax (used by `gan expand`, GEP-0002-R008).

use crate::ast::{Call, Callee, StrPart, Term};

pub fn print_module(term: &Term) -> String {
    let mut out = String::new();
    for stmt in term.as_block() {
        print_stmt(&stmt, 0, &mut out);
        out.push('\n');
    }
    out
}

fn indent(level: usize, out: &mut String) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

fn print_stmt(term: &Term, level: usize, out: &mut String) {
    indent(level, out);
    print_term(term, level, out);
}

const BLOCK_FORMS: &[&str] = &[
    "defmodule", "def", "defp", "defmacro", "if", "unless", "case", "cond", "quote", "with",
    "for", "fn",
];

fn is_operator(name: &str) -> bool {
    matches!(
        name,
        "+" | "-"
            | "*"
            | "/"
            | "//"
            | "=="
            | "!="
            | "<"
            | ">"
            | "<="
            | ">="
            | "and"
            | "or"
            | "="
            | "++"
            | "<>"
            | ".."
            | "|>"
            | "when"
            | "<-"
            | "|"
    )
}

fn print_term(term: &Term, level: usize, out: &mut String) {
    match term {
        Term::Int(n) => out.push_str(&n.to_string()),
        Term::Float(f) => {
            let s = format!("{f}");
            out.push_str(&s);
            if !s.contains('.') && !s.contains('e') {
                out.push_str(".0");
            }
        }
        Term::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Term::Nil => out.push_str("nil"),
        Term::Str(parts) => {
            out.push('"');
            for p in parts {
                match p {
                    StrPart::Text(t) => {
                        for c in t.chars() {
                            match c {
                                '\n' => out.push_str("\\n"),
                                '\t' => out.push_str("\\t"),
                                '"' => out.push_str("\\\""),
                                '\\' => out.push_str("\\\\"),
                                '#' => out.push_str("\\#"),
                                other => out.push(other),
                            }
                        }
                    }
                    StrPart::Interp(e) => {
                        out.push_str("#{");
                        print_term(e, level, out);
                        out.push('}');
                    }
                }
            }
            out.push('"');
        }
        Term::Atom(a) => {
            if a.chars().all(|c| c.is_alphanumeric() || c == '_') && !a.is_empty() {
                out.push(':');
                out.push_str(a);
            } else {
                out.push_str(&format!(":\"{a}\""));
            }
        }
        Term::Var(name, ctx) => {
            out.push_str(name);
            if let Some(id) = ctx {
                out.push_str(&format!("__gan{id}"));
            }
        }
        Term::Alias(segs) => out.push_str(&segs.join(".")),
        Term::List(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                print_term(item, level, out);
            }
            out.push(']');
        }
        Term::Tuple(items) => {
            out.push('{');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                print_term(item, level, out);
            }
            out.push('}');
        }
        Term::Map(entries) => {
            out.push_str("%{");
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                match k {
                    Term::Atom(a) if a.chars().all(|c| c.is_alphanumeric() || c == '_') => {
                        out.push_str(a);
                        out.push_str(": ");
                    }
                    other => {
                        print_term(other, level, out);
                        out.push_str(" => ");
                    }
                }
                print_term(v, level, out);
            }
            out.push('}');
        }
        Term::Pair(k, v) => {
            out.push_str(k);
            out.push_str(": ");
            print_term(v, level, out);
        }
        Term::Call(call) => print_call(call, level, out),
    }
}

fn print_call(call: &Call, level: usize, out: &mut String) {
    match &call.callee {
        Callee::Name(name) => {
            if name == "%struct%" || name == "%struct_update%" || name == "%map_update%" {
                let mut idx = 0;
                out.push('%');
                if name != "%map_update%" {
                    print_term(&call.args[idx], level, out);
                    idx += 1;
                }
                out.push('{');
                if name != "%struct%" {
                    print_term(&call.args[idx], level, out);
                    idx += 1;
                    out.push_str(" | ");
                }
                if let Term::List(pairs) = &call.args[idx] {
                    for (i, p) in pairs.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        print_term(p, level, out);
                    }
                }
                out.push('}');
                return;
            }
            if name == "__block__" {
                for (i, stmt) in call.args.iter().enumerate() {
                    if i > 0 {
                        out.push('\n');
                        indent(level, out);
                    }
                    print_term(stmt, level, out);
                }
                return;
            }
            if is_operator(name) && call.args.len() == 2 {
                print_operand(&call.args[0], level, out, name);
                out.push(' ');
                out.push_str(name);
                out.push(' ');
                print_operand(&call.args[1], level, out, name);
                return;
            }
            if (name == "-" || name == "not" || name == "^" || name == "&") && call.args.len() == 1
            {
                out.push_str(name);
                if name == "not" {
                    out.push(' ');
                }
                let needs_paren = matches!(&call.args[0], Term::Call(c)
                    if matches!(&c.callee, Callee::Name(n) if is_operator(n)));
                if needs_paren {
                    out.push('(');
                }
                print_term(&call.args[0], level, out);
                if needs_paren {
                    out.push(')');
                }
                return;
            }
            if name.starts_with('@') {
                out.push_str(name);
                if !call.args.is_empty() {
                    out.push(' ');
                    print_args(&call.args, level, out, false);
                }
                return;
            }
            let has_block = Term::keyword_arg(&call.args, "do").is_some();
            if has_block && BLOCK_FORMS.contains(&name.as_str()) {
                print_block_form(name, call, level, out);
                return;
            }
            out.push_str(name);
            out.push('(');
            print_args(&call.args, level, out, false);
            out.push(')');
        }
        Callee::Dot {
            base,
            name,
            is_call,
        } => {
            let needs_paren = matches!(**base, Term::Call(ref c)
                if matches!(&c.callee, Callee::Name(n) if is_operator(n)));
            if needs_paren {
                out.push('(');
            }
            print_term(base, level, out);
            if needs_paren {
                out.push(')');
            }
            out.push('.');
            out.push_str(name);
            if *is_call {
                out.push('(');
                print_args(&call.args, level, out, false);
                out.push(')');
            }
        }
        Callee::Apply(f) => {
            print_term(f, level, out);
            out.push_str(".(");
            print_args(&call.args, level, out, false);
            out.push(')');
        }
    }
}

fn op_prec(name: &str) -> u8 {
    match name {
        "=" => 1,
        "when" => 2,
        "<-" => 3,
        "or" => 4,
        "and" => 5,
        "==" | "!=" => 6,
        "<" | ">" | "<=" | ">=" => 7,
        "|>" => 8,
        "++" | "<>" => 9,
        ".." => 10,
        "+" | "-" => 11,
        "*" | "/" | "//" => 12,
        _ => 13,
    }
}

fn print_operand(term: &Term, level: usize, out: &mut String, parent_op: &str) {
    let needs_paren = matches!(term, Term::Call(c)
        if matches!(&c.callee, Callee::Name(n)
            if is_operator(n) && c.args.len() == 2 && op_prec(n) < op_prec(parent_op)));
    if needs_paren {
        out.push('(');
    }
    print_term(term, level, out);
    if needs_paren {
        out.push(')');
    }
}

fn print_args(args: &[Term], level: usize, out: &mut String, _cmd: bool) {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        print_term(arg, level, out);
    }
}

fn print_block_form(name: &str, call: &Call, level: usize, out: &mut String) {
    out.push_str(name);
    let plain: Vec<&Term> = call
        .args
        .iter()
        .filter(|a| !matches!(a, Term::Pair(k, _) if k == "do" || k == "else"))
        .collect();
    if !plain.is_empty() {
        out.push(' ');
        for (i, arg) in plain.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            print_term(arg, level, out);
        }
    }
    out.push_str(" do");
    if let Some(body) = Term::keyword_arg(&call.args, "do") {
        print_section(body, level + 1, out);
    }
    if let Some(body) = Term::keyword_arg(&call.args, "else") {
        out.push('\n');
        indent(level, out);
        out.push_str("else");
        print_section(body, level + 1, out);
    }
    out.push('\n');
    indent(level, out);
    out.push_str("end");
}

fn print_section(body: &Term, level: usize, out: &mut String) {
    if body.is_call_named("__clauses__") {
        if let Term::Call(c) = body {
            for clause in &c.args {
                if let Term::Call(cl) = clause {
                    out.push('\n');
                    indent(level, out);
                    if let Term::List(pats) = &cl.args[0] {
                        for (i, p) in pats.iter().enumerate() {
                            if i > 0 {
                                out.push_str(", ");
                            }
                            print_term(p, level, out);
                        }
                    }
                    out.push_str(" ->");
                    for stmt in cl.args[1].as_block() {
                        out.push('\n');
                        print_stmt(&stmt, level + 1, out);
                    }
                }
            }
        }
        return;
    }
    for stmt in body.as_block() {
        out.push('\n');
        print_stmt(&stmt, level, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_expr_str;

    fn roundtrip(src: &str) -> String {
        let t = parse_expr_str("<test>", src).unwrap();
        let mut out = String::new();
        print_term(&t, 0, &mut out);
        out
    }

    #[test]
    fn prints_expressions() {
        assert_eq!(roundtrip("1 + 2 * 3"), "1 + 2 * 3");
        assert_eq!(roundtrip("%{a: 1}"), "%{a: 1}");
        assert_eq!(roundtrip("[1, 2, 3]"), "[1, 2, 3]");
        assert_eq!(roundtrip(":math.sqrt(2.0)"), ":math.sqrt(2.0)");
        assert_eq!(roundtrip("\"hi #{name}\""), "\"hi #{name}\"");
    }

    #[test]
    fn prints_block_forms() {
        let printed = roundtrip("if x do\n  1\nelse\n  2\nend");
        assert_eq!(printed, "if x do\n  1\nelse\n  2\nend");
        let printed = roundtrip("case x do\n  :ok -> 1\nend");
        assert_eq!(printed, "case x do\n  :ok ->\n    1\nend");
    }
}
