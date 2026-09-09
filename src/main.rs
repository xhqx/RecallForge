//! Command-line client; JSON output is suitable for scripts and agent tool calls.
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use recallforge::service::Service;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long, env = "RECALLFORGE_DATA_DIR", global = true)]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Init,
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    Save {
        #[arg(long)]
        project: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        lexical_only: bool,
    },
    Search {
        #[arg(long)]
        project: String,
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "hybrid")]
        mode: String,
        #[arg(long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        include_inactive: bool,
    },
    Get {
        #[arg(long)]
        project: String,
        #[arg(long)]
        id: String,
    },
    History {
        #[arg(long)]
        project: String,
        #[arg(long)]
        id: String,
    },
    Neighbors {
        #[arg(long)]
        project: String,
        #[arg(long)]
        id: String,
    },
    Link {
        #[arg(long)]
        project: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        relation: String,
    },
    Reindex {
        #[arg(long)]
        project: String,
    },
    Export {
        #[arg(long)]
        project: String,
        #[arg(long, default_value = "json")]
        format: String,
    },
    Backup {
        #[arg(long)]
        output: PathBuf,
    },
    Doctor,
    Mcp,
}
#[derive(Subcommand)]
enum ProjectCommand {
    Add {
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        root: PathBuf,
    },
    List,
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    let dir = cli
        .data_dir
        .or_else(|| {
            directories::ProjectDirs::from("dev", "RecallForge", "RecallForge")
                .map(|d| d.data_local_dir().to_path_buf())
        })
        .context("cannot locate data directory; supply --data-dir")?;
    let mut service = Service::open(dir)?;
    let result:Value=match cli.command {
        Command::Mcp=>return recallforge::mcp::serve(&mut service),
        Command::Init=>json!({"data_dir":service.data_dir,"ready":true}),
        Command::Project{command:ProjectCommand::Add{id,name,root}}=>serde_json::to_value(service.store.add_project(&id,&name,&root)?)?,
        Command::Project{command:ProjectCommand::List}=>service.call("project_list",json!({}))?,
        Command::Save{project,file,lexical_only}=>{let content=std::fs::read(file)?;anyhow::ensure!(content.len()<=64_000,"input exceeds 64 KB");let entry:Value=serde_json::from_slice(&content)?;service.call("memory_save",json!({"project":project,"entry":entry,"lexical_only":lexical_only}))?},
        Command::Search{project,query,mode,limit,include_inactive}=>service.call("memory_search",json!({"project":project,"query":query,"mode":mode,"limit":limit,"include_inactive":include_inactive}))?,
        Command::Get{project,id}=>service.call("memory_get",json!({"project":project,"id":id}))?,
        Command::History{project,id}=>service.call("memory_history",json!({"project":project,"id":id}))?,
        Command::Neighbors{project,id}=>service.call("memory_neighbors",json!({"project":project,"id":id}))?,
        Command::Link{project,from,to,relation}=>service.call("memory_link",json!({"project":project,"from":from,"to":to,"relation":relation}))?,
        Command::Reindex{project}=>service.call("memory_reindex",json!({"project":project}))?,
        Command::Export{project,format}=>service.call("memory_export",json!({"project":project,"format":format}))?,
        Command::Backup{output}=>{service.backup(&output)?;json!({"backup":output})},
        Command::Doctor=>service.call("memory_doctor",json!({}))?,
    };
    if let Some(text) = result.get("markdown").and_then(Value::as_str) {
        print!("{text}");
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(())
}
