use codespan_reporting::Diagnostic;
use std::io;
use std::io::prelude::*;

use crate::core;

pub fn compile_module(
    writer: &mut impl Write,
    module: &core::Module,
) -> io::Result<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");

    writeln!(writer, "<!--")?;
    writeln!(
        writer,
        "  This file is automatically @generated by {} {}",
        pkg_name, pkg_version,
    )?;
    writeln!(writer, "  It is not intended for manual editing.")?;
    writeln!(writer, "-->")?;

    for item in &module.items {
        match item {
            core::Item::Struct {
                span: _,
                doc,
                name,
                fields,
            } => {
                writeln!(writer)?;
                writeln!(writer, "## {}", name)?;

                if !doc.is_empty() {
                    writeln!(writer)?;
                    writeln!(writer, "{}", doc)?;
                }

                if !fields.is_empty() {
                    writeln!(writer)?;
                    writeln!(writer, "### Fields")?;
                    writeln!(writer)?;
                    writeln!(writer, "| Name | Type |")?;
                    writeln!(writer, "| ---- | ---- |")?;

                    for (_field_doc, _, field_name, field_ty) in fields {
                        write!(writer, "| {} | ", field_name)?;
                        compile_ty(writer, field_ty, &mut diagnostics)?;
                        writeln!(writer, " |")?;
                    }

                    // TODO: output field docs
                }
            }
        }
    }

    Ok(diagnostics)
}

fn compile_ty(
    writer: &mut impl Write,
    term: &core::Term,
    _diagnostics: &mut Vec<Diagnostic>,
) -> io::Result<()> {
    match term {
        core::Term::U8(_) => write!(writer, "U8"),
        core::Term::Error(_) => write!(writer, "**invalid data description**"),
    }
}
