use super::{version_output, Shell, ShellConfiguration};

pub struct Powershell;

impl Shell for Powershell {
    fn friendly_name(&self) -> &'static str {
        "PowerShell"
    }

    fn app_alias(&self, alias_name: &str) -> String {
        format!(
            "function {} {{\n\
             \x20   $history = (Get-History -Count 1).CommandLine;\n\
             \x20   if (-not [string]::IsNullOrWhiteSpace($history)) {{\n\
             \x20       $fuck = $(thefuck $args $history);\n\
             \x20       if (-not [string]::IsNullOrWhiteSpace($fuck)) {{\n\
             \x20           if ($fuck.StartsWith(\"echo\")) {{ $fuck = $fuck.Substring(5); }}\n\
             \x20           else {{ iex \"$fuck\"; }}\n\
             \x20       }}\n\
             \x20   }}\n\
             \x20   [Console]::ResetColor() \n\
             }}\n",
            alias_name
        )
    }

    fn and_(&self, commands: &[&str]) -> String {
        commands
            .iter()
            .map(|command| format!("({})", command))
            .collect::<Vec<_>>()
            .join(" -and ")
    }

    fn how_to_configure(&self) -> Option<ShellConfiguration> {
        Some(ShellConfiguration {
            content: "iex \"$(thefuck --alias)\"".to_string(),
            path: "$profile".to_string(),
            reload: ". $profile".to_string(),
            can_configure_automatically: false,
        })
    }

    fn version(&self) -> String {
        let version = version_output("powershell.exe", &["$PSVersionTable.PSVersion"]);
        match version.trim_end().split('\n').next_back() {
            Some(last_line) if !last_line.trim().is_empty() => {
                last_line.split_whitespace().collect::<Vec<_>>().join(".")
            }
            _ => version_output("pwsh", &["--version"])
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_commands() {
        assert_eq!(Powershell.and_(&["ls", "cd"]), "(ls) -and (cd)");
        assert_eq!(Powershell.or_(&["ls", "cd"]), "ls || cd");
    }

    #[test]
    fn generates_alias() {
        let alias = Powershell.app_alias("fuck");
        assert!(alias
            .starts_with("function fuck {\n    $history = (Get-History -Count 1).CommandLine;\n"));
        assert!(alias.contains("        $fuck = $(thefuck $args $history);\n"));
        assert!(alias.ends_with("    [Console]::ResetColor() \n}\n"));
    }
}
