use rtodo::workspace::Workspace;
use rtodo::cli::CLI;
use rtodo::dispatch::exec_cmd;
use clap::Parser;



fn main() {

    let mut workspace = match Workspace::load_or_init() {
        Ok(w) => w,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };
    let cli = CLI::parse();

    if let Err(msg) = exec_cmd(cli.command, &mut workspace) {
        eprintln!("{}", msg);
        std::process::exit(1);
    }

    if let Err(msg) = workspace.save() {
        eprintln!("error writing the workspace {}", msg);
        std::process::exit(1)
    }

}
