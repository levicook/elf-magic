# 📖 Usage Guide

**Using your generated ELF constants in practice**

Once elf-magic generates your ELF constants, here's how to use them effectively in different scenarios.

## Basic Usage

After building your ELF crate, you'll have constants for each Solana program:

```rust
use my_elves::{TOKEN_MANAGER_ELF, GOVERNANCE_ELF, elves};

// Use individual constants
let program_data = TOKEN_MANAGER_ELF;
println!("Token manager program is {} bytes", program_data.len());

// Or iterate through all programs
for (name, elf_data) in elves() {
    println!("Program '{}' is {} bytes", name, elf_data.len());
}
```

## Testing

### Unit Tests

```rust
use my_elves::TOKEN_MANAGER_ELF;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_loads() {
        // Verify ELF data is valid
        assert!(TOKEN_MANAGER_ELF.len() > 0);
        assert_eq!(TOKEN_MANAGER_ELF[0..4], [0x7f, b'E', b'L', b'F']);
    }
}
```

### Integration Tests with Test Validator

```rust
use my_elves::{TOKEN_MANAGER_ELF, GOVERNANCE_ELF};
use solana_test_validator::TestValidator;
use solana_client::rpc_client::RpcClient;

#[tokio::test]
async fn test_with_validator() -> Result<(), Box<dyn std::error::Error>> {
    let test_validator = TestValidator::with_no_fees(
        solana_sdk::pubkey::Pubkey::new_unique(),
        None,
        (),
    );

    let client = RpcClient::new(&test_validator.rpc_url());

    // Deploy your programs
    let token_program_id = deploy_program(&client, TOKEN_MANAGER_ELF).await?;
    let governance_program_id = deploy_program(&client, GOVERNANCE_ELF).await?;

    // Your integration tests here...

    Ok(())
}
```

## Deployment Scripts

### Simple Deployment

```rust
use my_elves::{TOKEN_MANAGER_ELF, GOVERNANCE_ELF};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::Signer;

async fn deploy_programs(client: &RpcClient, payer: &dyn Signer) -> Result<(), Error> {
    println!("Deploying programs...");

    let token_program_id = deploy_program(client, payer, TOKEN_MANAGER_ELF).await?;
    println!("Token Manager deployed at: {}", token_program_id);

    let governance_program_id = deploy_program(client, payer, GOVERNANCE_ELF).await?;
    println!("Governance deployed at: {}", governance_program_id);

    Ok(())
}
```

### Advanced Deployment with Dependencies

```rust
use my_elves::elves;
use std::collections::HashMap;

async fn deploy_with_dependencies(client: &RpcClient, payer: &dyn Signer) -> Result<DeploymentResult, Error> {
    let mut deployed_programs = HashMap::new();

    // Deploy in dependency order
    let deployment_order = ["spl_token", "my_vault", "my_dex", "my_governance"];

    for program_name in deployment_order {
        if let Some((_, elf_data)) = elves().into_iter().find(|(name, _)| name == &program_name) {
            let program_id = deploy_program(client, payer, elf_data).await?;
            deployed_programs.insert(program_name.to_string(), program_id);
            println!("✅ {} deployed at: {}", program_name, program_id);
        }
    }

    Ok(DeploymentResult { deployed_programs })
}
```

## Environment-Specific Deployment

### Devnet Deployment

```rust
use my_elves::elves;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = RpcClient::new("https://api.devnet.solana.com");

    let payer = read_keypair_file("~/.config/solana/id.json")?;

    for (name, elf_data) in elves() {
        match deploy_program(&client, &payer, elf_data).await {
            Ok(program_id) => println!("✅ {} -> {}", name, program_id),
            Err(e) => eprintln!("❌ {} failed: {}", name, e),
        }
    }

    Ok(())
}
```

### Mainnet Deployment with Confirmation

```rust
use my_elves::elves;

async fn deploy_to_mainnet() -> Result<(), Error> {
    println!("🚨 MAINNET DEPLOYMENT");
    println!("Programs to deploy:");

    for (name, elf_data) in elves() {
        println!("  - {} ({} bytes)", name, elf_data.len());
    }

    print!("Continue? (yes/no): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "yes" {
        println!("Deployment cancelled");
        return Ok(());
    }

    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    // ... proceed with deployment

    Ok(())
}
```

## Client Integration

### Web Application

```rust
// In your web server or API
use my_elves::{TOKEN_MANAGER_ELF, GOVERNANCE_ELF};
use warp::Filter;

#[tokio::main]
async fn main() {
    let programs = warp::path("programs")
        .map(|| {
            serde_json::json!({
                "token_manager": {
                    "size": TOKEN_MANAGER_ELF.len(),
                    "hash": sha256(TOKEN_MANAGER_ELF)
                },
                "governance": {
                    "size": GOVERNANCE_ELF.len(),
                    "hash": sha256(GOVERNANCE_ELF)
                }
            })
        });

    let routes = programs;

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

### CLI Tool

```rust
use my_elves::elves;
use clap::{App, Arg, SubCommand};

fn main() -> Result<(), Error> {
    let matches = App::new("my-cli")
        .subcommand(
            SubCommand::with_name("programs")
                .about("List available programs")
        )
        .subcommand(
            SubCommand::with_name("deploy")
                .about("Deploy programs")
                .arg(Arg::with_name("program")
                     .help("Program name to deploy")
                     .takes_value(true))
        )
        .get_matches();

    match matches.subcommand() {
        ("programs", _) => {
            println!("Available programs:");
            for (name, elf_data) in elves() {
                println!("  {} ({} bytes)", name, elf_data.len());
            }
        }
        ("deploy", Some(sub_m)) => {
            if let Some(program_name) = sub_m.value_of("program") {
                deploy_specific_program(program_name)?;
            } else {
                deploy_all_programs()?;
            }
        }
        _ => eprintln!("Unknown command"),
    }

    Ok(())
}
```

## Development Workflow

### Rapid Iteration

```rust
// In your development scripts
use my_elves::elves;

fn main() -> Result<(), Error> {
    // Start local validator
    let validator = start_test_validator()?;

    // Deploy all programs
    for (name, elf_data) in elves() {
        let program_id = deploy_program_local(elf_data)?;
        println!("📦 {} deployed locally at {}", name, program_id);
    }

    // Run your tests
    run_integration_tests()?;

    println!("🎉 Development cycle complete!");
    Ok(())
}
```

### Hot Reloading

```rust
use my_elves::elves;
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;

fn watch_and_redeploy() -> Result<(), Error> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;

    // Watch for program changes
    watcher.watch("programs/", RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(_) => {
                println!("🔄 Programs changed, redeploying...");

                // Rebuild (elf-magic will detect changes)
                std::process::Command::new("cargo")
                    .args(&["build"])
                    .status()?;

                // Redeploy
                for (name, elf_data) in elves() {
                    redeploy_program(name, elf_data)?;
                }

                println!("✅ Redeployment complete");
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
}
```

## Error Handling

### Robust Deployment

```rust
use my_elves::elves;

#[derive(Debug)]
struct DeploymentError {
    program: String,
    error: String,
}

async fn deploy_with_error_handling() -> Result<Vec<String>, Vec<DeploymentError>> {
    let mut successful = Vec::new();
    let mut failed = Vec::new();

    for (name, elf_data) in elves() {
        match deploy_program_with_retries(name, elf_data, 3).await {
            Ok(program_id) => {
                successful.push(format!("✅ {} -> {}", name, program_id));
            }
            Err(e) => {
                failed.push(DeploymentError {
                    program: name.to_string(),
                    error: e.to_string(),
                });
            }
        }
    }

    if failed.is_empty() {
        Ok(successful)
    } else {
        Err(failed)
    }
}
```

## Best Practices

### 1. Validate ELF Data

Always check that your ELF data is valid:

```rust
use my_elves::TOKEN_MANAGER_ELF;

fn validate_elf(elf_data: &[u8]) -> Result<(), &'static str> {
    if elf_data.len() < 4 {
        return Err("ELF too short");
    }

    if &elf_data[0..4] != &[0x7f, b'E', b'L', b'F'] {
        return Err("Invalid ELF magic");
    }

    Ok(())
}

// Always validate before deployment
validate_elf(TOKEN_MANAGER_ELF)?;
```

### 2. Use Constants for Program IDs

When deploying to known addresses:

```rust
use solana_sdk::pubkey;

pub const TOKEN_MANAGER_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111112");

// Deploy to specific address if needed
deploy_program_to_address(TOKEN_MANAGER_ELF, TOKEN_MANAGER_PROGRAM_ID)?;
```

### 3. Environment Configuration

```rust
use my_elves::elves;

struct Config {
    rpc_url: String,
    keypair_path: String,
    programs_to_deploy: Vec<String>,
}

impl Config {
    fn from_env() -> Self {
        Self {
            rpc_url: env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "http://localhost:8899".to_string()),
            keypair_path: env::var("SOLANA_KEYPAIR").unwrap_or_else(|_| "~/.config/solana/id.json".to_string()),
            programs_to_deploy: env::var("PROGRAMS")
                .unwrap_or_else(|_| "all".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}
```

### 4. Logging and Monitoring

```rust
use my_elves::elves;
use log::{info, warn, error};

async fn deploy_with_logging() -> Result<(), Error> {
    info!("Starting deployment of {} programs", elves().len());

    for (name, elf_data) in elves() {
        info!("Deploying {} ({} bytes)", name, elf_data.len());

        match deploy_program(elf_data).await {
            Ok(program_id) => {
                info!("✅ {} deployed successfully at {}", name, program_id);
            }
            Err(e) => {
                error!("❌ {} deployment failed: {}", name, e);
                return Err(e);
            }
        }
    }

    info!("🎉 All programs deployed successfully");
    Ok(())
}
```

## Integration Examples

### With Anchor

```rust
use my_elves::MY_PROGRAM_ELF;
use anchor_client::solana_sdk::pubkey::Pubkey;

#[tokio::test]
async fn test_anchor_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Deploy your program
    let program_id = deploy_program(MY_PROGRAM_ELF).await?;

    // Use with Anchor client
    let client = anchor_client::Client::new_with_options(
        cluster,
        Rc::new(payer),
        CommitmentConfig::processed(),
    );

    let program = client.program(program_id);

    // Your Anchor calls here...

    Ok(())
}
```

### With Seahorse

```rust
use my_elves::MY_SEAHORSE_PROGRAM_ELF;

// Deploy Seahorse-compiled program
let program_id = deploy_program(MY_SEAHORSE_PROGRAM_ELF).await?;
```

This usage guide covers the most common patterns for working with your generated ELF constants. The key is that once elf-magic generates your constants, they're just regular Rust `&[u8]` data that you can use however you need!
