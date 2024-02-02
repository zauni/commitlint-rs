mod parser;

use config::Config;
use miette::{miette, LabeledSpan, Report, Result};
use parser::{parse_commit, Commit};
use serde::Deserialize;

fn rule_scope_enum(commit: &Commit, opts: &ScopeEnumOpts) -> Option<Report> {
    let severity = &opts.0;
    let condition = &opts.1;
    let scopes = &opts.2;

    if severity == &Severity::Off {
        return None;
    }

    if let Some(scope) = &commit.scope {
        let is_in_scopes = scopes.contains(&scope.to_string());
        let is_valid = match condition {
            Condition::Never => !is_in_scopes,
            Condition::Always => is_in_scopes,
        };
        if !is_valid {
            return Some(
                miette!(
                    severity = match severity {
                        Severity::Warning => miette::Severity::Warning,
                        Severity::Error => miette::Severity::Error,
                        Severity::Off => miette::Severity::Advice,
                    },
                    labels = vec![LabeledSpan::at(
                        scope.start()..scope.end(),
                        "not allowed scope"
                    ),],
                    help = String::from("scope must") + match condition {
                        Condition::Never => " not",
                        Condition::Always => "",
                    } + " be one of " + &scopes.join(", "),
                    code = "rule/scope-enum",
                    url = "https://example.com",
                    "Scope not allowed",
                )
                .with_source_code(commit.raw.clone()),
            );
        }
    }

    None
}

/// Severity of the rule
#[derive(Debug, Deserialize, PartialEq)]
enum Severity {
    /// Turn off the rule
    #[serde(rename = "off")]
    Off,
    /// Warn about the violation of a rule
    #[serde(rename = "warning")]
    Warning,
    /// Error about the violation of a rule
    #[serde(rename = "error")]
    Error,
}

/// When the rule should be applied
#[derive(Debug, Deserialize)]
enum Condition {
    /// The options should "never" be found (e.g. in a list of disallowed values)
    #[serde(rename = "never")]
    Never,
    /// The options should "always" be found (e.g. in a list of allowed values)
    #[serde(rename = "always")]
    Always,
}

/// Options for the scope-enum rule
type ScopeEnumOpts = (Severity, Condition, Vec<String>);

/// Config all the rules
#[derive(Debug, Deserialize)]
struct RulesDetails {
    #[serde(rename = "scope-enum")]
    scope_enum: ScopeEnumOpts,
}

/// Config
#[derive(Debug, Deserialize)]
struct RulesConfig {
    rules: RulesDetails,
}

fn main() -> Result<()> {
    let commit_message =
        "feat(nice): add cool feature\n\nsome body\n\nsecond body line\n\nsome footer";

    let commit = parse_commit(&commit_message);
    println!("{:#?}", commit);

    let settings = Config::builder()
        // Source can be `commitlint.config.toml` or `commitlint.config.json
        .add_source(config::File::with_name("src/commitlint.config"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `COMMITLINT_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("COMMITLINT"))
        .build()
        .unwrap();

    // Print out our settings
    let config: RulesConfig = settings.try_deserialize::<RulesConfig>().unwrap();
    println!("{:?}", config);

    if let Some(report) = rule_scope_enum(&commit, &config.rules.scope_enum) {
        return Err(report);
    }

    Ok(())
}
