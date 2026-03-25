use colored::Colorize;

use crate::types::{GeneratedFlake, ProjectInfo};

/// Print a rich colored summary after flake generation.
pub fn print_report(project: &ProjectInfo, flake: &GeneratedFlake) {
    println!();
    println!("{}", "=== nixify report ===".green().bold());
    println!();

    // Detected languages and build systems
    println!("{}", "Detected:".cyan().bold());
    for lang_info in &project.languages {
        println!(
            "  {} {} ({})",
            "->".green(),
            lang_info.language,
            lang_info.build_system,
        );
    }
    println!();

    // Generated flake info
    println!("{}", "Generated:".cyan().bold());
    println!("  {} flake style: {}", "->".green(), flake.style);
    if !flake.inputs_used.is_empty() {
        println!(
            "  {} inputs: {}",
            "->".green(),
            flake.inputs_used.join(", ")
        );
    }
    println!();

    // Dev shell packages
    if !flake.devshell_packages.is_empty() {
        println!("{}", "Dev shell packages:".cyan().bold());
        for pkg in &flake.devshell_packages {
            println!("  {} {}", "->".green(), pkg);
        }
        println!();
    }

    // Checks
    if !flake.checks.is_empty() {
        println!("{}", "CI checks:".cyan().bold());
        for check in &flake.checks {
            println!("  {} {}", "->".green(), check);
        }
        println!();
    }

    // Next steps
    println!("{}", "Next steps:".yellow().bold());
    println!(
        "  {} Enter the dev shell:       {}",
        "1.".white().bold(),
        "nix develop".cyan()
    );
    println!(
        "  {} Build the project:         {}",
        "2.".white().bold(),
        "nix build".cyan()
    );
    println!(
        "  {} Run flake checks:          {}",
        "3.".white().bold(),
        "nix flake check".cyan()
    );
    println!(
        "  {} Commit flake.nix to git:   {}",
        "4.".white().bold(),
        "git add flake.nix flake.lock".cyan()
    );
    println!();

    // Beginner tips
    println!("{}", "Tips:".blue().bold());
    println!(
        "  {} If you use direnv, add {} to auto-enter the shell.",
        "*".blue(),
        "use flake".cyan()
    );
    println!(
        "  {} Customize your dev shell by editing {} or {}.",
        "*".blue(),
        ".nixify.toml".cyan(),
        "flake.nix".cyan()
    );
    println!(
        "  {} Run {} to re-generate after changing config.",
        "*".blue(),
        "nixify init".cyan()
    );
    println!();
}
