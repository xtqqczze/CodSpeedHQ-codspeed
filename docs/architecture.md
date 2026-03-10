# Architecture

## Execution flow

This diagram shows how the CLI commands, project config, orchestrator (with providers), and executors interact during a benchmark run.

```mermaid
graph TD
    subgraph "1. CLI Commands"
        RUN["codspeed run &lt;command&gt;"]
        EXEC["codspeed exec &lt;command&gt;"]
    end

    subgraph "2. Config"
        PROJ_CFG["ProjectConfig<br/>(codspeed.yaml in repo)<br/>benchmark targets, defaults"]
        MERGER["ConfigMerger<br/>CLI args > project config > defaults"]
    end

    subgraph "3. Orchestrator"
        ORCH_CFG["OrchestratorConfig<br/>targets, modes, upload settings"]
        ORCH["Orchestrator"]

        PROVIDER{" "}
        LOCAL["LocalProvider"]
        CI["CI Providers<br/>(GitHub Actions, GitLab, Buildkite)"]
        PROVIDER_JOIN{" "}

        subgraph "Executor (per mode × per target)"
            SETUP["1. Setup"]
            RUN_STEP["2. Run"]
            TEARDOWN["3. Teardown"]
        end

        UPLOAD["Upload all results to CodSpeed"]
    end

    subgraph "4. Auth"
        CS_CFG["CodSpeedConfig<br/>(~/.config/codspeed/config.yaml)"]
        OIDC["OIDC / env token"]
    end

    %% CLI → Config → OrchestratorConfig
    RUN --> MERGER
    EXEC --> MERGER
    MERGER --> PROJ_CFG
    PROJ_CFG -->|"merged config"| ORCH_CFG

    %% CLI → Orchestrator
    RUN -->|"single command →<br/>Entrypoint target"| ORCH_CFG
    RUN -->|"no command + config →<br/>Exec & Entrypoint targets"| ORCH_CFG
    EXEC -->|"always creates<br/>Exec target"| ORCH_CFG

    %% Orchestrator init
    ORCH_CFG -->|"Orchestrator::new()"| ORCH

    %% Provider detection
    ORCH -->|"auto-detect env"| PROVIDER
    PROVIDER --> LOCAL
    PROVIDER --> CI

    %% Auth → Providers
    CS_CFG -->|"auth token"| LOCAL
    OIDC -->|"OIDC / env token"| CI

    %% Providers → Upload
    LOCAL -->|"token + run metadata"| PROVIDER_JOIN{" "}
    CI -->|"token + run metadata"| PROVIDER_JOIN
    PROVIDER_JOIN --> UPLOAD

    %% Orchestrator spawns executors
    ORCH -->|"for each target × mode:<br/>spawn executor"| SETUP
    SETUP --> RUN_STEP
    RUN_STEP --> TEARDOWN

    %% All executors done → upload
    TEARDOWN -->|"collect results"| UPLOAD
```

### Key interactions

- **CLI → Config**: Both `run` and `exec` merge CLI args with `ProjectConfig` (CLI takes precedence). `run` can source targets from project config; `exec` always creates an `Exec` target.
- **CLI → Orchestrator**: The merged config becomes an `OrchestratorConfig` holding all targets and modes.
- **Orchestrator → Providers**: Auto-detects environment (Local vs CI). Local uses the auth token from `CodSpeedConfig`; CI providers handle OIDC tokens.
- **Orchestrator → Executors**: Groups all `Exec` targets into one exec-harness pipe command, runs each `Entrypoint` independently. For each target group, iterates over all modes, creating an `ExecutionContext` per mode and dispatching to the matching executor (`Valgrind`/`WallTime`/`Memory`). After all runs complete, uploads results with provider metadata.
