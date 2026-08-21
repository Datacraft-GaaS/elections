// eel-synth: deterministic synthetic data generator for the eel IEMS.
//
// Usage: eel-synth <SUBCOMMAND> [options]
// All generators accept --seed <u64>. Same seed produces byte-identical output.

use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use eel_synth::{
    embassies, forms, geo, kits, output, personnel, verify, voters, DegradationFlags, OutputFormat,
    ScaleProfile,
};

#[derive(Parser)]
#[command(
    name = "eel-synth",
    about = "Deterministic synthetic data generator for the eel election-management system",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate synthetic voter register records.
    Voters {
        #[arg(long, default_value = "42")]
        seed: u64,
        #[arg(long, default_value = "100000")]
        scale: u64,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value = "csv")]
        format: String,
    },

    /// Generate polling station, constituency, and county geometry.
    Geo {
        #[arg(long, default_value = "42")]
        seed: u64,
        #[arg(long, default_value = "4600")]
        stations: u64,
        #[arg(long)]
        out: Option<PathBuf>,
        /// Output format: geojson or sql
        #[arg(long, default_value = "geojson")]
        format: String,
    },

    /// Generate Form 34A images (PNG).
    Forms {
        #[arg(long, default_value = "42")]
        seed: u64,
        #[arg(long, default_value = "200")]
        scale: u64,
        /// Directory to write PNG files into.
        #[arg(long, default_value = "forms_out")]
        out: PathBuf,
        /// Comma-separated degradation flags: torn,wet,angled,low-light
        #[arg(long, default_value = "")]
        degraded: String,
        /// Number of polling stations to draw from (generates geo internally).
        #[arg(long, default_value = "100")]
        stations: u64,
    },

    /// Generate KIEMS kit records.
    Kits {
        #[arg(long, default_value = "42")]
        seed: u64,
        #[arg(long, default_value = "6500")]
        scale: u64,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value = "csv")]
        format: String,
        #[arg(long, default_value = "4600")]
        stations: u64,
    },

    /// Generate election personnel records.
    Personnel {
        #[arg(long, default_value = "42")]
        seed: u64,
        #[arg(long, default_value = "25000")]
        scale: u64,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value = "csv")]
        format: String,
        #[arg(long, default_value = "4600")]
        stations: u64,
    },

    /// Generate diaspora embassy voting locations.
    Embassies {
        #[arg(long, default_value = "42")]
        seed: u64,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value = "csv")]
        format: String,
    },

    /// Run all generators with a named scale profile.
    All {
        #[arg(long, default_value = "42")]
        seed: u64,
        /// Scale profile: default or smoke
        #[arg(long, default_value = "default")]
        scale: String,
        /// Output directory root.
        #[arg(long, default_value = "synth_out")]
        out: PathBuf,
        #[arg(long, default_value = "csv")]
        format: String,
    },

    /// Verify determinism: compute hash of default output, compare to checkpoint.
    Verify {
        #[arg(long, default_value = "42")]
        seed: u64,
        /// Scale profile: default or smoke
        #[arg(long, default_value = "default")]
        scale: String,
    },
}

fn progress_bar(len: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_message(msg.to_owned());
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    pb
}

fn parse_format(s: &str) -> Result<OutputFormat> {
    s.parse()
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Voters {
            seed,
            scale,
            out,
            format,
        } => {
            let fmt = parse_format(&format)?;
            if fmt == OutputFormat::Parquet {
                return Err(output::parquet_unsupported());
            }

            eprintln!("eel-synth voters: seed={} count={}", seed, scale);
            let geo_data = geo::generate(seed, (scale / 10).max(10))?;
            let pb = progress_bar(scale, "generating voters");
            let records = voters::generate(seed, scale, &geo_data.stations)?;
            pb.finish_with_message("done");

            let mut sink = output::OutputSink::open(out.as_deref())?;
            match fmt {
                OutputFormat::Csv => voters::write_csv(&records, &mut sink)?,
                OutputFormat::Sql => voters::write_sql_copy(&records, &mut sink)?,
                OutputFormat::Parquet => unreachable!(),
            }
        }

        Commands::Geo {
            seed,
            stations,
            out,
            format,
        } => {
            eprintln!("eel-synth geo: seed={} stations={}", seed, stations);
            let data = geo::generate(seed, stations)?;

            match format.to_ascii_lowercase().as_str() {
                "geojson" => {
                    let mut sink = output::OutputSink::open(out.as_deref())?;
                    let json = geo::to_geojson(&data.stations);
                    use std::io::Write;
                    sink.write_all(json.as_bytes())?;
                    sink.write_all(b"\n")?;
                }
                "sql" => {
                    let mut sink = output::OutputSink::open(out.as_deref())?;
                    geo::write_sql_copy(&data.stations, &data.constituencies, &mut sink)?;
                }
                other => {
                    anyhow::bail!("unsupported geo format '{}'. Use geojson or sql.", other);
                }
            }
        }

        Commands::Forms {
            seed,
            scale,
            out,
            degraded,
            stations,
        } => {
            let deg: DegradationFlags = degraded.parse()?;
            eprintln!(
                "eel-synth forms: seed={} count={} out={} degraded={:?}",
                seed,
                scale,
                out.display(),
                deg
            );
            let geo_data = geo::generate(seed, stations)?;
            let _records = forms::generate(seed, scale, &geo_data.stations, &deg, &out)?;
            eprintln!(
                "eel-synth forms: wrote {} PNG files to {}",
                scale,
                out.display()
            );
        }

        Commands::Kits {
            seed,
            scale,
            out,
            format,
            stations,
        } => {
            let fmt = parse_format(&format)?;
            if fmt == OutputFormat::Parquet {
                return Err(output::parquet_unsupported());
            }
            eprintln!("eel-synth kits: seed={} count={}", seed, scale);
            let geo_data = geo::generate(seed, stations)?;
            let kit_list = kits::generate(seed, scale, &geo_data.stations)?;

            let mut sink = output::OutputSink::open(out.as_deref())?;
            match fmt {
                OutputFormat::Csv => kits::write_csv(&kit_list, &mut sink)?,
                OutputFormat::Sql => kits::write_sql_copy(&kit_list, &mut sink)?,
                OutputFormat::Parquet => unreachable!(),
            }
        }

        Commands::Personnel {
            seed,
            scale,
            out,
            format,
            stations,
        } => {
            let fmt = parse_format(&format)?;
            if fmt == OutputFormat::Parquet {
                return Err(output::parquet_unsupported());
            }
            eprintln!("eel-synth personnel: seed={} count={}", seed, scale);
            let geo_data = geo::generate(seed, stations)?;
            let p = personnel::generate(seed, scale, &geo_data.stations)?;

            let mut sink = output::OutputSink::open(out.as_deref())?;
            match fmt {
                OutputFormat::Csv => personnel::write_csv(&p, &mut sink)?,
                OutputFormat::Sql => personnel::write_sql_copy(&p, &mut sink)?,
                OutputFormat::Parquet => unreachable!(),
            }
        }

        Commands::Embassies {
            seed: _,
            out,
            format,
        } => {
            let fmt = parse_format(&format)?;
            if fmt == OutputFormat::Parquet {
                return Err(output::parquet_unsupported());
            }
            let e = embassies::load_all()?;
            let mut sink = output::OutputSink::open(out.as_deref())?;
            match fmt {
                OutputFormat::Csv => embassies::write_csv(&e, &mut sink)?,
                OutputFormat::Sql => embassies::write_sql_copy(&e, &mut sink)?,
                OutputFormat::Parquet => unreachable!(),
            }
        }

        Commands::All {
            seed,
            scale,
            out,
            format,
        } => {
            let profile: ScaleProfile = scale.parse()?;
            let fmt = parse_format(&format)?;
            if fmt == OutputFormat::Parquet {
                return Err(output::parquet_unsupported());
            }

            eprintln!("eel-synth all: seed={} profile={:?}", seed, profile);
            std::fs::create_dir_all(&out)?;

            // Geo.
            eprintln!("  [1/5] geo ({} stations)...", profile.stations);
            let geo_data = geo::generate(seed, profile.stations)?;
            let geo_out = out.join("stations.geojson");
            let geo_json = geo::to_geojson(&geo_data.stations);
            std::fs::write(&geo_out, geo_json)?;
            let sql_out = out.join("geo.sql");
            let mut sf = std::fs::File::create(&sql_out)?;
            geo::write_sql_copy(&geo_data.stations, &geo_data.constituencies, &mut sf)?;

            let ext = match fmt {
                OutputFormat::Csv => "csv",
                OutputFormat::Sql => "sql",
                OutputFormat::Parquet => unreachable!(),
            };

            // Voters.
            eprintln!("  [2/5] voters ({})...", profile.voters);
            let voter_records = voters::generate(seed, profile.voters, &geo_data.stations)?;
            {
                let path = out.join(format!("voters.{}", ext));
                let mut f = std::fs::File::create(&path)?;
                match fmt {
                    OutputFormat::Csv => voters::write_csv(&voter_records, &mut f)?,
                    OutputFormat::Sql => voters::write_sql_copy(&voter_records, &mut f)?,
                    _ => unreachable!(),
                }
            }

            // Kits.
            eprintln!("  [3/5] kits ({})...", profile.kits);
            let kit_list = kits::generate(seed, profile.kits, &geo_data.stations)?;
            {
                let path = out.join(format!("kits.{}", ext));
                let mut f = std::fs::File::create(&path)?;
                match fmt {
                    OutputFormat::Csv => kits::write_csv(&kit_list, &mut f)?,
                    OutputFormat::Sql => kits::write_sql_copy(&kit_list, &mut f)?,
                    _ => unreachable!(),
                }
            }

            // Personnel.
            eprintln!("  [4/5] personnel ({})...", profile.personnel);
            let p = personnel::generate(seed, profile.personnel, &geo_data.stations)?;
            {
                let path = out.join(format!("personnel.{}", ext));
                let mut f = std::fs::File::create(&path)?;
                match fmt {
                    OutputFormat::Csv => personnel::write_csv(&p, &mut f)?,
                    OutputFormat::Sql => personnel::write_sql_copy(&p, &mut f)?,
                    _ => unreachable!(),
                }
            }

            // Embassies.
            eprintln!("  [5/5] embassies...");
            let e_list = embassies::generate(profile.embassies)?;
            {
                let path = out.join(format!("embassies.{}", ext));
                let mut f = std::fs::File::create(&path)?;
                match fmt {
                    OutputFormat::Csv => embassies::write_csv(&e_list, &mut f)?,
                    OutputFormat::Sql => embassies::write_sql_copy(&e_list, &mut f)?,
                    _ => unreachable!(),
                }
            }

            eprintln!("eel-synth all: complete. Output in {}", out.display());
        }

        Commands::Verify { seed, scale } => {
            let profile: ScaleProfile = scale.parse()?;
            verify::run(seed, &profile)?;
        }
    }

    Ok(())
}
