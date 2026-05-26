use clap::{Parser, Subcommand};
use nocode_core::{
    build_project, check_project, write_rust, BuildOutput, BuildProjectParams, BuildTarget,
    CheckOutput, CompileCache, PipelineError,
};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Parser)]
#[command(
    name = "qp",
    about = "Quantum Point — visual graphs → universal IR → target build"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Universal check: graph → IR / domain (no emit, no cargo)
    Check {
        #[arg(default_value = "examples/hello-rust/graphs/main.qp")]
        project: PathBuf,
    },
    /// Alias for Check (same as Studio ▶ Run)
    Run {
        #[arg(default_value = "examples/hello-rust/graphs/main.qp")]
        project: PathBuf,
    },
    /// Emit artifacts + target toolchain (Core/Rust: cargo build)
    Build {
        #[arg(default_value = "examples/hello-rust/graphs/main.qp")]
        project: PathBuf,
        #[arg(short, long, default_value = ".nocode/build/rust")]
        out: PathBuf,
        #[arg(long, default_value = "dev")]
        profile: String,
    },
    /// Build + run binary (Core/Rust: cargo run)
    Exec {
        #[arg(default_value = "examples/hello-rust/graphs/main.qp")]
        project: PathBuf,
        #[arg(short, long, default_value = ".nocode/build/rust")]
        out: PathBuf,
        #[arg(long, default_value = "dev")]
        profile: String,
    },
    /// Emit Rust sources only (no cargo)
    Emit {
        #[arg(default_value = "examples/hello-rust/graphs/main.qp")]
        project: PathBuf,
        #[arg(short, long, default_value = ".nocode/build/rust")]
        out: PathBuf,
    },
}

#[derive(Debug, Error)]
enum CliError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph file: {0}")]
    GraphFile(#[from] graph_model::GraphFileParseError),
    #[error("pipeline: {0}")]
    Pipeline(#[from] PipelineError),
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { project } | Commands::Run { project } => {
            let proj = load_qp(&project)?;
            let root = project_root_for_qp(&project);
            let mut cache = CompileCache::default();
            let out = check_project(&proj, &mut cache, &[], Some(&root))?;
            print_check(&out);
        }
        Commands::Build {
            project,
            out,
            profile,
        } => {
            let proj = load_qp(&project)?;
            let root = project_root_for_qp(&project);
            let out = resolve_build_out(&project, &out, &proj);
            let target = BuildTarget::default_for_layer(proj.layer);
            let mut cache = CompileCache::default();
            let result = build_project(&mut BuildProjectParams {
                project: &proj,
                project_root: &root,
                out_dir: &out,
                profile: &profile,
                target,
                cache: &mut cache,
                dirty_nodes: &[],
                run_after_build: false,
            })?;
            print_build(&result);
            if !result.success {
                std::process::exit(result.exit_code);
            }
            println!("built → {}", out.display());
        }
        Commands::Exec {
            project,
            out,
            profile,
        } => {
            let proj = load_qp(&project)?;
            let root = project_root_for_qp(&project);
            let out = resolve_build_out(&project, &out, &proj);
            let target = BuildTarget::default_for_layer(proj.layer);
            let mut cache = CompileCache::default();
            let result = build_project(&mut BuildProjectParams {
                project: &proj,
                project_root: &root,
                out_dir: &out,
                profile: &profile,
                target,
                cache: &mut cache,
                dirty_nodes: &[],
                run_after_build: true,
            })?;
            print_build(&result);
            if !result.success {
                std::process::exit(result.exit_code);
            }
        }
        Commands::Emit { project, out } => {
            let proj = load_qp(&project)?;
            let program = nocode_core::compile_project(&proj)?;
            write_rust(&out, &program)?;
            println!("emitted → {}", out.display());
        }
    }
    Ok(())
}

fn resolve_build_out(
    qp_path: &std::path::Path,
    out: &PathBuf,
    proj: &graph_model::Project,
) -> PathBuf {
    let root = project_root_for_qp(qp_path);
    let target = BuildTarget::default_for_layer(proj.layer);
    let under_project = nocode_core::resolve_build_dir(&root, target);
    let norm = out.to_string_lossy().replace('\\', "/");
    if norm == ".nocode/build/rust" || norm.ends_with("/.nocode/build/rust") {
        return to_absolute_path(&under_project);
    }
    if out.is_absolute() {
        out.clone()
    } else {
        to_absolute_path(&root.join(out))
    }
}

fn to_absolute_path(path: &std::path::Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn project_root_for_qp(qp_path: &std::path::Path) -> PathBuf {
    if let Some(parent) = qp_path.parent() {
        if parent.file_name().and_then(|n| n.to_str()) == Some("graphs") {
            return parent
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| parent.to_path_buf());
        }
        return parent.to_path_buf();
    }
    PathBuf::from(".")
}

fn load_qp(path: &PathBuf) -> Result<graph_model::Project, CliError> {
    let raw = std::fs::read(path)?;
    Ok(graph_model::Project::from_qp_bytes(&raw)?)
}

fn print_check(out: &CheckOutput) {
    println!("{}", out.summary);
    if !out.success {
        std::process::exit(1);
    }
}

fn print_build(result: &BuildOutput) {
    if !result.stderr.is_empty() {
        eprint!("{}", result.stderr);
    }
    if !result.stdout.is_empty() {
        print!("{}", result.stdout);
    }
}
