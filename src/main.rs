mod version;
mod launch;
mod java;
 
use version::AnyError;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
    )
    .init();

    tracing::info!("Nexus Launcher Starting...");
    version::utils::init_workspace()?;

    // Suppose we set a dedicated Java storage directory for the launcher
    let custom_runtime_dir = version::utils::get_minecraft_dir().join("runtimes");
    
    // Start scanning (pass in the custom directory from earlier)
    let local_javas = java::scan_local_java_environments(Some(&custom_runtime_dir)).await;
    
    for java in &local_javas {
        tracing::info!(
            "📦 Found Java {} (full version: {}) -> Path: {}", 
            java.major_version, 
            java.full_version, 
            java.path.display()
        );
    }

    let manifest = version::source::obtain_manifest().await?;

    let target_version = "1.20.1";
    if let Some(v_info) = manifest.versions.iter().find(|v| v.id == target_version) {
        
        tracing::info!("Parsing data of {}...", target_version);
        let detail = version::source::fetch_version_detail(&v_info.url).await?;

        let client_jar_path = version::utils::get_minecraft_dir()
            .join("versions")
            .join(target_version)
            .join(format!("{}.jar", target_version));
        
        if !client_jar_path.exists() {
            tracing::info!("Downloading core files...");
            version::download::download_and_verify(&detail.downloads.client.url, &client_jar_path, detail.downloads.client.sha1.as_str()).await?;
        }

        // let classpath_libs = version::source::download_libraries(&detail).await?;

        tracing::info!("\nAll core components of {} are ready!", target_version);
        tracing::info!("Core Path: {:?}", client_jar_path);

        version::source::download_assets(&detail).await?;
        // launch::start_game(&detail, &client_jar_path, classpath_libs, "AuroBreeze")?;
    }

    Ok(())
}
