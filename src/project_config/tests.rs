use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_deserialize_minimal_config() {
    let yaml = r#"
options:
  warmup-time: 1s
"#;
    let config: ProjectConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.options.is_some());
    let options = config.options.unwrap();
    assert!(options.walltime.is_some());
    assert_eq!(
        options.walltime.unwrap().warmup_time,
        Some("1s".to_string())
    );
}

#[test]
fn test_deserialize_full_walltime_config() {
    let yaml = r#"
options:
  warmup-time: 2s
  max-time: 10s
  min-time: 1s
  max-rounds: 100
  min-rounds: 10
  working-directory: ./bench
"#;
    let config: ProjectConfig = serde_yaml::from_str(yaml).unwrap();
    let options = config.options.unwrap();
    let walltime = options.walltime.unwrap();

    assert_eq!(walltime.warmup_time, Some("2s".to_string()));
    assert_eq!(walltime.max_time, Some("10s".to_string()));
    assert_eq!(walltime.min_time, Some("1s".to_string()));
    assert_eq!(walltime.max_rounds, Some(100));
    assert_eq!(walltime.min_rounds, Some(10));
    assert_eq!(options.working_directory, Some("./bench".to_string()));
}

#[test]
fn test_deserialize_empty_config() {
    let yaml = r#"{}"#;
    let config: ProjectConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.options.is_none());
}

#[test]
fn test_validate_conflicting_min_time_max_rounds() {
    let config = ProjectConfig {
        options: Some(ProjectOptions {
            walltime: Some(WalltimeOptions {
                warmup_time: None,
                max_time: None,
                min_time: Some("1s".to_string()),
                max_rounds: Some(10),
                min_rounds: None,
            }),
            working_directory: None,
        }),
        benchmarks: None,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("cannot use both min_time and max_rounds")
    );
}

#[test]
fn test_validate_conflicting_max_time_min_rounds() {
    let config = ProjectConfig {
        options: Some(ProjectOptions {
            walltime: Some(WalltimeOptions {
                warmup_time: None,
                max_time: Some("10s".to_string()),
                min_time: None,
                max_rounds: None,
                min_rounds: Some(5),
            }),
            working_directory: None,
        }),
        benchmarks: None,
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("cannot use both max_time and min_rounds")
    );
}

#[test]
fn test_validate_valid_config() {
    let config = ProjectConfig {
        options: Some(ProjectOptions {
            walltime: Some(WalltimeOptions {
                warmup_time: Some("1s".to_string()),
                max_time: Some("10s".to_string()),
                min_time: Some("2s".to_string()),
                max_rounds: None,
                min_rounds: None,
            }),
            working_directory: Some("./bench".to_string()),
        }),
        benchmarks: None,
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_load_from_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("codspeed.yaml");

    fs::write(
        &config_path,
        r#"
options:
  warmup-time: 5s
"#,
    )
    .unwrap();

    let config = ProjectConfig::load_from_path(&config_path).unwrap();
    assert!(config.options.is_some());
}

#[test]
fn test_load_from_path_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("codspeed.yaml");

    fs::write(&config_path, "invalid: yaml: content:").unwrap();

    let result = ProjectConfig::load_from_path(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_discover_with_explicit_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("my-config.yaml");

    fs::write(
        &config_path,
        r#"
options:
  warmup-time: 3s
"#,
    )
    .unwrap();

    let discovered =
        DiscoveredProjectConfig::discover_and_load(Some(&config_path), temp_dir.path()).unwrap();

    assert!(discovered.is_some());
    let discovered = discovered.unwrap();
    assert!(discovered.config.options.is_some());
}

#[test]
fn test_discover_with_explicit_path_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("missing.yaml");

    let result = DiscoveredProjectConfig::discover_and_load(Some(&config_path), temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_discover_finds_codspeed_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("codspeed.yaml");

    fs::write(
        &config_path,
        r#"
options:
  warmup-time: 2s
"#,
    )
    .unwrap();

    let discovered = DiscoveredProjectConfig::discover_and_load(None, temp_dir.path()).unwrap();

    assert!(discovered.is_some());
}

#[test]
fn test_discover_priority_yaml_over_yml() {
    let temp_dir = TempDir::new().unwrap();

    // Create both .yaml and .yml files
    fs::write(
        temp_dir.path().join("codspeed.yaml"),
        r#"
options:
  warmup-time: 1s
"#,
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("codspeed.yml"),
        r#"
options:
  warmup-time: 2s
"#,
    )
    .unwrap();

    let discovered = DiscoveredProjectConfig::discover_and_load(None, temp_dir.path()).unwrap();

    assert!(discovered.is_some());
    let discovered = discovered.unwrap();
    // Verify the .yaml file was picked over .yml (priority order)
    assert!(discovered.path.ends_with("codspeed.yaml"));
}

#[test]
fn test_discover_no_config_found() {
    let temp_dir = TempDir::new().unwrap();
    let discovered = DiscoveredProjectConfig::discover_and_load(None, temp_dir.path()).unwrap();
    assert!(discovered.is_none());
}

#[test]
fn test_deserialize_exec_target() {
    let yaml = r#"
benchmarks:
  - exec: ls -al /nix/store
  - name: my exec benchmark
    exec: ./my_binary
    options:
      warmup-time: 1s
"#;
    let config: ProjectConfig = serde_yaml::from_str(yaml).unwrap();
    let benchmarks = config.benchmarks.unwrap();
    assert_eq!(benchmarks.len(), 2);

    assert_eq!(
        benchmarks[0].command,
        TargetCommand::Exec {
            exec: "ls -al /nix/store".to_string()
        }
    );
    assert!(benchmarks[0].name.is_none());

    assert_eq!(
        benchmarks[1].command,
        TargetCommand::Exec {
            exec: "./my_binary".to_string()
        }
    );
    assert_eq!(benchmarks[1].name, Some("my exec benchmark".to_string()));
    let walltime = benchmarks[1]
        .options
        .as_ref()
        .unwrap()
        .walltime
        .as_ref()
        .unwrap();
    assert_eq!(walltime.warmup_time, Some("1s".to_string()));
}

#[test]
fn test_deserialize_entrypoint_target() {
    let yaml = r#"
benchmarks:
  - name: my python benchmarks
    entrypoint: pytest --codspeed src
  - entrypoint: cargo bench
"#;
    let config: ProjectConfig = serde_yaml::from_str(yaml).unwrap();
    let benchmarks = config.benchmarks.unwrap();
    assert_eq!(benchmarks.len(), 2);

    assert_eq!(
        benchmarks[0].command,
        TargetCommand::Entrypoint {
            entrypoint: "pytest --codspeed src".to_string()
        }
    );
    assert_eq!(benchmarks[0].name, Some("my python benchmarks".to_string()));

    assert_eq!(
        benchmarks[1].command,
        TargetCommand::Entrypoint {
            entrypoint: "cargo bench".to_string()
        }
    );
    assert!(benchmarks[1].name.is_none());
}

#[test]
fn test_deserialize_mixed_targets() {
    let yaml = r#"
benchmarks:
  - exec: ls -al
  - entrypoint: pytest --codspeed src
"#;
    let config: ProjectConfig = serde_yaml::from_str(yaml).unwrap();
    let benchmarks = config.benchmarks.unwrap();
    assert_eq!(benchmarks.len(), 2);
    assert!(matches!(benchmarks[0].command, TargetCommand::Exec { .. }));
    assert!(matches!(
        benchmarks[1].command,
        TargetCommand::Entrypoint { .. }
    ));
}

#[test]
fn test_deserialize_target_missing_exec_and_entrypoint() {
    let yaml = r#"
benchmarks:
  - name: missing command
"#;
    let result: Result<ProjectConfig, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err());
}

#[test]
fn test_deserialize_target_both_exec_and_entrypoint() {
    let yaml = r#"
benchmarks:
  - exec: ls
    entrypoint: pytest
"#;
    let result: Result<ProjectConfig, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("a target cannot have both `exec` and `entrypoint`")
    );
}
