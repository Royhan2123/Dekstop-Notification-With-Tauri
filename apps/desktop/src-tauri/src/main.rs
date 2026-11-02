// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Dipanggil git lewat hook `pre-push`; tidak menjalankan aplikasi.
    if args.get(1).map(String::as_str) == Some("hook") {
        std::process::exit(isla_desk_lib::interface::cli::run_hook(&args[2..]));
    }
    isla_desk_lib::run()
}
