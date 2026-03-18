use exec_harness::walltime::WalltimeExecutionArgs;

use super::WalltimeOptions;

/// Handles merging of CLI arguments with project configuration
///
/// Implements the precedence rule: CLI > config > None
pub struct ConfigMerger;

impl ConfigMerger {
    /// Merge walltime execution args with project config walltime options
    ///
    /// CLI arguments take precedence over config values. If a CLI arg is None
    /// and a config value exists, the config value is used.
    pub fn merge_walltime_options(
        cli: &WalltimeExecutionArgs,
        config_opts: Option<&WalltimeOptions>,
    ) -> WalltimeExecutionArgs {
        WalltimeExecutionArgs {
            warmup_time: Self::merge_option(
                &cli.warmup_time,
                config_opts.and_then(|c| c.warmup_time.as_ref()),
            ),
            max_time: Self::merge_option(
                &cli.max_time,
                config_opts.and_then(|c| c.max_time.as_ref()),
            ),
            min_time: Self::merge_option(
                &cli.min_time,
                config_opts.and_then(|c| c.min_time.as_ref()),
            ),
            max_rounds: cli.max_rounds.or(config_opts.and_then(|c| c.max_rounds)),
            min_rounds: cli.min_rounds.or(config_opts.and_then(|c| c.min_rounds)),
        }
    }

    /// Helper to merge Option values with precedence: CLI > config > None
    fn merge_option<T: Clone>(cli_value: &Option<T>, config_value: Option<&T>) -> Option<T> {
        cli_value.clone().or_else(|| config_value.cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_walltime_all_from_cli() {
        let cli = WalltimeExecutionArgs {
            warmup_time: Some("5s".to_string()),
            max_time: Some("20s".to_string()),
            min_time: None,
            max_rounds: Some(50),
            min_rounds: None,
        };

        let config = WalltimeOptions {
            warmup_time: Some("1s".to_string()),
            max_time: Some("10s".to_string()),
            min_time: Some("2s".to_string()),
            max_rounds: Some(100),
            min_rounds: Some(10),
        };

        let merged = ConfigMerger::merge_walltime_options(&cli, Some(&config));

        // CLI values should win
        assert_eq!(merged.warmup_time, Some("5s".to_string()));
        assert_eq!(merged.max_time, Some("20s".to_string()));
        // Config values should be used when CLI is None
        assert_eq!(merged.min_time, Some("2s".to_string()));
        assert_eq!(merged.max_rounds, Some(50));
        assert_eq!(merged.min_rounds, Some(10));
    }

    #[test]
    fn test_merge_walltime_all_from_config() {
        let cli = WalltimeExecutionArgs {
            warmup_time: None,
            max_time: None,
            min_time: None,
            max_rounds: None,
            min_rounds: None,
        };

        let config = WalltimeOptions {
            warmup_time: Some("3s".to_string()),
            max_time: Some("15s".to_string()),
            min_time: None,
            max_rounds: Some(200),
            min_rounds: None,
        };

        let merged = ConfigMerger::merge_walltime_options(&cli, Some(&config));

        // All from config
        assert_eq!(merged.warmup_time, Some("3s".to_string()));
        assert_eq!(merged.max_time, Some("15s".to_string()));
        assert_eq!(merged.min_time, None);
        assert_eq!(merged.max_rounds, Some(200));
        assert_eq!(merged.min_rounds, None);
    }

    #[test]
    fn test_merge_walltime_no_config() {
        let cli = WalltimeExecutionArgs {
            warmup_time: Some("2s".to_string()),
            max_time: None,
            min_time: None,
            max_rounds: Some(30),
            min_rounds: None,
        };

        let merged = ConfigMerger::merge_walltime_options(&cli, None);

        // Should be same as CLI
        assert_eq!(merged.warmup_time, Some("2s".to_string()));
        assert_eq!(merged.max_time, None);
        assert_eq!(merged.min_time, None);
        assert_eq!(merged.max_rounds, Some(30));
        assert_eq!(merged.min_rounds, None);
    }

    #[test]
    fn test_merge_option_helper() {
        // CLI value wins
        let cli_val = Some("cli".to_string());
        let config_val = Some("config".to_string());
        let result = ConfigMerger::merge_option(&cli_val, config_val.as_ref());
        assert_eq!(result, Some("cli".to_string()));

        // Config value used when CLI is None
        let cli_val: Option<String> = None;
        let config_val = Some("config".to_string());
        let result = ConfigMerger::merge_option(&cli_val, config_val.as_ref());
        assert_eq!(result, Some("config".to_string()));

        // Both None
        let cli_val: Option<String> = None;
        let config_val: Option<String> = None;
        let result = ConfigMerger::merge_option(&cli_val, config_val.as_ref());
        assert_eq!(result, None);
    }
}
