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

    Lastfm {
        #[arg(long, short)]
        similar: String,
    },
}

pub fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Files { r#move: true, .. } => todo!(),
        Commands::Files {
            transcode: true, ..
        } => {
            // 11k, all skip: 0.2 s (rust), 0.6 s (python)
            use crate::io::SOURCE;
            use crate::transcode::SourceDir;
            SourceDir::new(&SOURCE).unwrap().transcode_all().unwrap();
        }
        Commands::Lastfm { similar: artist } => {
            let mut t = crate::lastfm::ArtistTree::new(&artist);
            t.build();
            t.as_dot(crate::lastfm::DotOutput::Svg).unwrap();
        }
        _ => unimplemented!(),
    }
}
