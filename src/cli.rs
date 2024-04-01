use clap::Parser;
use clap::Subcommand;

// discogs --artist=<name>
// discogs --collection --browse [tui]
// discogs --collection --dump
// discogs --collection --search=<query>
// discogs --release=<id>
// discogs --search --artist=<artist> --album=<album>
// files <--move|--transcode>
// lastfm --similar=<artist>
// library --dump
// tagger [tui]

// https://github.com/clap-rs/clap/blob/9d14f394ba22f65f8957310a03ae5fd613f89d76/examples/git-derive.rs
// https://github.com/atuinsh/atuin/blob/82a7c8d3219749dd298b23bae22456657ee92575/atuin/src/command/client/history.rs#L33

#[derive(Debug, Parser)]
#[command(name = "dita")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
#[command(infer_subcommands = true)] // important!
enum Commands {
    #[clap(group(
    clap::ArgGroup::new("foo")
        .required(true)
        .args(&["move", "transcode"]),
    ))]
    Files {
        #[clap(action)]
        #[arg(long, short)]
        r#move: bool,
        #[clap(action)]
        #[arg(long, short)]
        transcode: bool,
    },
}

pub fn main() {
    let args = Cli::parse();
    println!("{:?}", args);
}
