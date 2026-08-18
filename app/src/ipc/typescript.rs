use super::contract::descriptors;
use super::shape::Shape;
use super::types::shapes;

const HEAD: &str = "\
// Сгенерировано из контракта IPC: `cargo run -p tolearn-app --example ipc-types`.
// Руками не править — тест `типы_для_ui_совпадают_с_файлом_в_репозитории` сверяет байты.

export type IpcError = { code: string; message: string };
";

pub fn emit() -> String {
    let mut out = HEAD.to_owned();
    for shape in shapes() {
        out.push('\n');
        out.push_str(&declared(&shape));
    }
    out.push_str("\nexport type Commands = {\n");
    for descriptor in descriptors() {
        out.push_str(&format!(
            "  {}: {{ input: {}; output: {} }};\n",
            descriptor.name, descriptor.input.name, descriptor.output.name
        ));
    }
    out.push_str("};\n\nexport type CommandName = keyof Commands;\n");
    out
}

fn declared(shape: &Shape) -> String {
    let mut out = format!("export type {} = {{\n", shape.name);
    for field in &shape.fields {
        out.push_str(&format!("  {}: {};\n", field.name, field.ty));
    }
    out.push_str("};\n");
    out
}
