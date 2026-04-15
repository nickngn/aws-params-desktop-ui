fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set("FileDescription", "AWS Secrets & Parameters Manager");
        res.set("ProductName", "AWS Param UI");
        res.set("CompanyName", "NickNgn");
        res.set("LegalCopyright", "Copyright (c) 2026 NickNgn");
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.compile().expect("Failed to compile Windows resources");
    }
}
