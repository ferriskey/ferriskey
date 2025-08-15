use ferriskey_operator::crd::cluster::FerriskeyCluster;
use kube::CustomResourceExt;
use std::fs;
use std::path::Path;

fn main() {
    let crd = FerriskeyCluster::crd();
    let bytes = serde_yaml::to_string(&crd).unwrap();

    let dir_path = Path::new("crds");
    if !dir_path.exists() {
        fs::create_dir_all(dir_path).unwrap_or_else(|e| {
            eprintln!(
                "❌ Erreur lors de la création du répertoire {}: {}",
                dir_path.display(),
                e
            );
            std::process::exit(1);
        });

        println!("✅ Répertoire créé : {}", dir_path.display());
    }

    std::fs::write("crds/crd-ferriskeycluster.yaml", bytes).unwrap_or_else(|e| {
        eprintln!("Failed to write file: {}", e);
        std::process::exit(1);
    });

    println!("✅ CRD YAML générée : crd-ferriskeycluster.yaml");
}
