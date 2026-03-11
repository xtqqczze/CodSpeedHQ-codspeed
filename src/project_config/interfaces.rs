use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// Project-level configuration from codspeed.yaml file
///
/// This configuration provides default options for the run and exec commands.
/// CLI arguments always take precedence over config file values.
#[derive(Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectConfig {
    /// Default options to apply to all benchmark runs
    pub options: Option<ProjectOptions>,
    /// List of benchmark targets to execute
    pub benchmarks: Option<Vec<Target>>,
}

/// A benchmark target to execute.
///
/// Either `exec` or `entrypoint` must be specified (mutually exclusive).
#[derive(Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Target {
    /// Optional name for this target (display purposes only)
    pub name: Option<String>,
    /// Optional id to run a subset of targets (e.g. `codspeed run --bench my_id`)
    pub id: Option<String>,
    /// The command to run
    #[serde(flatten)]
    pub command: TargetCommand,
    /// Target-specific options
    pub options: Option<TargetOptions>,
}

/// The command for a benchmark target — exactly one of `exec` or `entrypoint`.
#[derive(Debug, Clone, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TargetCommand {
    /// Command measured by exec-harness
    Exec { exec: String },
    /// Command with built-in benchmark harness
    Entrypoint { entrypoint: String },
}

#[derive(Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct TargetOptions {
    #[serde(flatten)]
    pub walltime: Option<WalltimeOptions>,
}

/// Root-level options that apply to all benchmark runs unless overridden by CLI
#[derive(Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectOptions {
    /// Working directory where commands will be executed (relative to config file)
    pub working_directory: Option<String>,
    /// Walltime execution configuration (flattened)
    #[serde(flatten)]
    pub walltime: Option<WalltimeOptions>,
}

/// Walltime execution options matching WalltimeExecutionArgs structure
#[derive(Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WalltimeOptions {
    /// Duration of warmup phase (e.g., "1s", "500ms")
    pub warmup_time: Option<String>,
    /// Maximum total execution time
    pub max_time: Option<String>,
    /// Minimum total execution time
    pub min_time: Option<String>,
    /// Maximum number of rounds
    pub max_rounds: Option<u64>,
    /// Minimum number of rounds
    pub min_rounds: Option<u64>,
}

// Custom implementation to enforce mutual exclusivity of `exec` and `entrypoint` fields, not
// directly supported by serde's untagged enums.
impl<'de> Deserialize<'de> for TargetCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct RawTarget {
            exec: Option<String>,
            entrypoint: Option<String>,
        }

        let raw = RawTarget::deserialize(deserializer)?;
        Ok(match (raw.exec, raw.entrypoint) {
            (Some(exec), None) => TargetCommand::Exec { exec },
            (None, Some(entrypoint)) => TargetCommand::Entrypoint { entrypoint },
            (Some(_), Some(_)) => {
                return Err(serde::de::Error::custom(
                    "a target cannot have both `exec` and `entrypoint`",
                ));
            }
            (None, None) => {
                return Err(serde::de::Error::custom(
                    "a target must have either `exec` or `entrypoint`",
                ));
            }
        })
    }
}
