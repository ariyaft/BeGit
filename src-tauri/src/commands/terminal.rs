use crate::models::TerminalCommandResult;
use crate::utils::git_command;

fn parse_git_arguments(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for character in input.trim().chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }

        match character {
            '\\' if quote != Some('\'') => escaped = true,
            '\'' | '"' if quote == Some(character) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(character),
            whitespace if whitespace.is_whitespace() && quote.is_none() => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }

    if quote.is_some() {
        return Err("Unclosed quote in Git command.".to_string());
    }
    if escaped {
        current.push('\\');
    }
    if !current.is_empty() {
        args.push(current);
    }
    if args.first().is_some_and(|arg| arg.eq_ignore_ascii_case("git")) {
        args.remove(0);
    }
    if args.is_empty() {
        return Err("Enter a Git command, for example: status or log --oneline.".to_string());
    }

    Ok(args)
}

#[tauri::command]
pub fn run_git_terminal_command(path: String, command: String) -> Result<TerminalCommandResult, String> {
    let args = parse_git_arguments(&command)?;
    let output = git_command()
        .args(&args)
        .current_dir(&path)
        .output()
        .map_err(|error| format!("Could not start Git: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = match (stdout.trim(), stderr.trim()) {
        ("", "") => "Command completed with no output.".to_string(),
        (out, "") => out.to_string(),
        ("", err) => err.to_string(),
        (out, err) => format!("{out}\n{err}"),
    };

    if output.status.success() {
        Ok(TerminalCommandResult { output: text })
    } else {
        Err(format!("Git exited with {}:\n{}", output.status.code().map_or("an unknown error".to_string(), |code| code.to_string()), text))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_git_arguments;

    #[test]
    fn parses_optional_git_prefix_and_quoted_arguments() {
        assert_eq!(
            parse_git_arguments("git commit -m \"A useful message\"").unwrap(),
            vec!["commit", "-m", "A useful message"]
        );
    }

    #[test]
    fn rejects_unclosed_quotes() {
        assert!(parse_git_arguments("log --grep='unfinished").is_err());
    }
}
