//! 把 7 个外壳区域的规格以 JSON 打到标准输出,供跨仓库一致性检查与 Dozer 的 `workspace.json` 比对。
//!   cargo run -q --example dump_shell_regions
fn main() {
    println!("{}", serde_json::to_string_pretty(&byteui::theme::region::spec_dump()).expect("spec serialises"));
}
