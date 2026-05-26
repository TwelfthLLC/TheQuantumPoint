use ir::{sanitize_ident, EnumDef, FunctionDef, StructDef};
use std::fmt::Write;

use crate::actions::emit_action;

pub(crate) fn emit_struct(out: &mut String, s: &StructDef) {
    let name = sanitize_ident(&s.name);
    writeln!(out, "#[derive(Debug, Clone)]").unwrap();
    write!(out, "struct {name} {{").unwrap();
    for (i, field) in s.fields.iter().enumerate() {
        if i > 0 {
            write!(out, ", ").unwrap();
        }
        let f = sanitize_ident(field);
        write!(out, " {f}: String").unwrap();
    }
    writeln!(out, " }}\n").unwrap();
}

pub(crate) fn emit_enum(out: &mut String, e: &EnumDef) {
    let name = sanitize_ident(&e.name);
    writeln!(out, "#[derive(Debug, Clone)]").unwrap();
    writeln!(out, "enum {name} {{").unwrap();
    for v in &e.variants {
        let variant = sanitize_ident(v);
        writeln!(out, "    {variant},").unwrap();
    }
    writeln!(out, "}}\n").unwrap();
}

pub(crate) fn emit_function(out: &mut String, f: &FunctionDef) {
    let name = sanitize_ident(&f.name);
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| format!("{}: i64", sanitize_ident(p)))
        .collect();
    writeln!(out, "fn {name}({}) {{", params.join(", ")).unwrap();
    for a in &f.body {
        emit_action(out, a, 1);
    }
    writeln!(out, "}}\n").unwrap();
}
