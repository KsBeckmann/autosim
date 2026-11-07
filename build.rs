fn main() {
    // Embute o ícone no executável do Windows; nos demais sistemas o
    // executável não carrega ícone e só a janela o exibe.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()
            .expect("falha ao embutir o ícone no executável");
    }
}
