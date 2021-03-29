// Esse script só faz com que o build seja triggerado
// quando o arquivo instructions.yml é mudado.

fn main() {
    println!("cargo:rerun-if-changed=instructions.yml");
}
