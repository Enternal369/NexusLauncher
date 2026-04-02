use super::models::{VersionManifest, VersionDetail};
use super::download;
use super::utils;
use tracing;
use super::AnyError;
use std::path::PathBuf;
use super::models::AssetIndexManifest;
use tokio::fs;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::task::JoinSet;

/// obtain_manifest
pub async fn obtain_manifest() -> Result<VersionManifest, AnyError> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    tracing::info!("Obtaining version manifest from {}", url);

    let response = reqwest::get(url).await?;

    let manifest = response.json::<VersionManifest>().await?;

    tracing::info!("Latest release: {}", manifest.latest.release);
    tracing::info!("Latest snapshot: {}", manifest.latest.snapshot);

    Ok(manifest)
}


/// fetch_version_detail
pub async fn fetch_version_detail(url: &str) -> Result<VersionDetail, AnyError> {
    tracing::trace!("Fetching version detail from {}", url);
    let response = reqwest::get(url).await?;
    let detail = response.json::<VersionDetail>().await?;
    tracing::trace!("Version detail: {:#?}", detail);
    Ok(detail)
}



pub async fn download_libraries(detail: &VersionDetail) -> Result<Vec<PathBuf>, AnyError> {
    tracing::info!("Start preparing the dependency library...");
    
    let mp = MultiProgress::new();
    
    let tasks: Vec<_> = detail.libraries.iter()
        .filter_map(|lib| lib.downloads.artifact.as_ref().map(|a| (lib.name.clone(), a.clone())))
        .collect();

    let main_pb = mp.add(ProgressBar::new(tasks.len() as u64));
    main_pb.set_style(ProgressStyle::with_template(
        " {spinner:.green} Overall progress: [{wide_bar:.green/white}] {pos}/{len} ({percent}%)"
    )?);

    let mut classpath_libs = Vec::new();
    let mut set = JoinSet::new();

    for (name, artifact) in tasks {
        let local_path = utils::get_library_path(&artifact.path);
        classpath_libs.push(local_path.clone());

        if !local_path.exists() {
            let mp_clone = mp.clone();
            set.spawn(async move {
                tracing::info!("Concurrent download dependencies: {}", name);
                
                let _ = mp_clone; 

                download::download_and_verify(&artifact.url, &local_path, &artifact.sha1).await
            });
        } else {
            main_pb.inc(1);
        }
    }

    while let Some(res) = set.join_next().await {
        res??; 
        main_pb.inc(1);
    }

    main_pb.finish_with_message("All dependent libraries are ready");
    Ok(classpath_libs)
}


pub async fn download_assets(detail: &VersionDetail) -> Result<(), AnyError> {
    tracing::info!("Start processing the asset files...");
    let mc_dir = utils::get_minecraft_dir();

    // 1. Download the Asset Index (for example, 1.20.json)
    let index_path = mc_dir
        .join("assets")
        .join("indexes")
        .join(format!("{}.json", detail.asset_index.id));

    if !index_path.exists() {
        tracing::info!("Downloading resource index: {}.json", detail.asset_index.id);
        download::download_and_verify(
            &detail.asset_index.url,
            &index_path,
            &detail.asset_index.sha1,
        ).await?;
    }

    // 2. Read and parse the Index
    let index_content = fs::read_to_string(&index_path).await?;
    let asset_manifest: AssetIndexManifest = serde_json::from_str(&index_content)?;
    
    // 3. Prepare for concurrent downloads
    let mp = MultiProgress::new();
    let tasks = asset_manifest.objects;
    let main_pb = mp.add(ProgressBar::new(tasks.len() as u64));
    main_pb.set_style(ProgressStyle::with_template(
        " {spinner:.yellow} Resource file: [{wide_bar:.yellow/white}] {pos}/{len} ({percent}%)"
    )?);

    let mut set = JoinSet::new();
    let objects_dir = mc_dir.join("assets").join("objects");

    // Set the maximum concurrency (recommended between 16 and 32; too high may cause disconnection, too low will slow down downloading)
    let max_concurrent = 32;

    for (_, object) in tasks {
        let base_url = "https://resources.download.minecraft.net";
        let hash = object.hash;
        let prefix = &hash[0..2];
        let local_path = objects_dir.join(prefix).join(&hash);
        let download_url = format!("{}/{}/{}", base_url, prefix, hash);

        if !local_path.exists() {
            // If the currently downloading tasks are full, wait for one to finish before adding a new one.
            while set.len() >= max_concurrent {
                if let Some(res) = set.join_next().await {
                    res??; // Check if the task that was just completed reported any errors
                    main_pb.inc(1); // Update the progress bar
                }
            }

            // Not full yet, or space has been freed up, continue launching!
            set.spawn(async move {
                download::download_and_verify(&download_url, &local_path, &hash).await
            });
        } else {
            main_pb.inc(1);
        }
    }

    // Handle the last remaining tasks that haven't been completed
    while let Some(res) = set.join_next().await {
        res??; 
        main_pb.inc(1);
    }

    main_pb.finish_with_message("All resource files are ready!");
    Ok(())
}
