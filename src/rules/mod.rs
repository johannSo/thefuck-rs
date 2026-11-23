//! The rules: every module describes one way of fixing a command.
//!
//! A rule consists of two functions: `match_` tells if the rule applies to
//! the command, `get_new_command` returns the fixed scripts, the best first.
//! Rules are free to panic on output they don't understand (and they index
//! into it without much ceremony), a panicked rule simply doesn't apply.
//!
//! To add a rule create a module, and add it to the `rules!` list below in
//! alphabetical order of the rule names.

use crate::types::Rule;

/// What rules usually need.
pub(crate) mod prelude {
    pub use crate::shells::shell;
    pub use crate::specific::archlinux::{archlinux_env, get_pkgfile};
    pub use crate::specific::brew::{brew_available, get_brew_path_prefix};
    pub use crate::specific::git::git_support;
    pub use crate::specific::npm::{get_scripts, npm_available};
    pub use crate::specific::sudo::sudo_support;
    pub use crate::specific::{apt_available, dnf_available, nix_available, yum_available};
    pub use crate::types::{never, Command, Rule};
    pub use crate::utils::*;

    /// Joins the commands, so the next one runs only if the previous succeeds.
    pub fn and_(commands: &[&str]) -> String {
        shell().and_(commands)
    }

    /// The only new command.
    pub fn one(script: &str) -> Vec<String> {
        vec![script.to_string()]
    }

    pub fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }
}

macro_rules! rules {
    ($($module:ident,)*) => {
        $(mod $module;)*

        /// Returns all rules in alphabetical order of their names.
        pub fn all() -> Vec<Rule> {
            vec![$($module::RULE,)*]
        }
    };
}

rules! {
    adb_unknown_command,
    ag_literal,
    apt_get,
    apt_get_search,
    apt_invalid_operation,
    apt_list_upgradable,
    apt_upgrade,
    aws_cli,
    az_cli,
    brew_cask_dependency,
    brew_install,
    brew_link,
    brew_reinstall,
    brew_uninstall,
    brew_unknown_command,
    brew_update_formula,
    cargo,
    cargo_no_command,
    cat_dir,
    cd_correction,
    cd_cs,
    cd_mkdir,
    cd_parent,
    chmod_x,
    choco_install,
    composer_not_command,
    conda_mistype,
    cp_create_destination,
    cp_omitting_directory,
    cpp11,
    dirty_untar,
    dirty_unzip,
    django_south_ghost,
    django_south_merge,
    dnf_no_such_command,
    docker_image_being_used_by_container,
    docker_login,
    docker_not_command,
    dry,
    fab_command_not_found,
    fix_alt_space,
    fix_file,
    gem_unknown_command,
    git_add,
    git_add_force,
    git_bisect_usage,
    git_branch_0flag,
    git_branch_delete,
    git_branch_delete_checked_out,
    git_branch_exists,
    git_branch_list,
    git_checkout,
    git_clone_git_clone,
    git_clone_missing,
    git_commit_add,
    git_commit_amend,
    git_commit_reset,
    git_diff_no_index,
    git_diff_staged,
    git_fix_stash,
    git_flag_after_filename,
    git_help_aliased,
    git_hook_bypass,
    git_lfs_mistype,
    git_main_master,
    git_merge,
    git_merge_unrelated,
    git_not_command,
    git_pull,
    git_pull_clone,
    git_pull_uncommitted_changes,
    git_push,
    git_push_different_branch_names,
    git_push_force,
    git_push_pull,
    git_push_without_commits,
    git_rebase_merge_dir,
    git_rebase_no_changes,
    git_remote_delete,
    git_remote_seturl_add,
    git_rm_local_modifications,
    git_rm_recursive,
    git_rm_staged,
    git_stash,
    git_stash_pop,
    git_tag_force,
    git_two_dashes,
    go_run,
    go_unknown_command,
    gradle_no_task,
    gradle_wrapper,
    grep_arguments_order,
    grep_recursive,
    grunt_task_not_found,
    gulp_not_task,
    has_exists_script,
    heroku_multiple_apps,
    heroku_not_command,
    history,
    hostscli,
    ifconfig_device_not_found,
    java,
    javac,
    lein_not_task,
    ln_no_hard_link,
    ln_s_order,
    long_form_help,
    ls_all,
    ls_lah,
    man,
    man_no_space,
    mercurial,
    missing_space_before_subcommand,
    mkdir_p,
    mvn_no_command,
    mvn_unknown_lifecycle_phase,
    nixos_cmd_not_found,
    no_command,
    no_such_file,
    npm_missing_script,
    npm_run_script,
    npm_wrong_command,
    omnienv_no_such_command,
    open,
    pacman,
    pacman_invalid_option,
    pacman_not_found,
    path_from_history,
    php_s,
    pip_install,
    pip_unknown_command,
    port_already_in_use,
    prove_recursively,
    python_command,
    python_execute,
    python_module_error,
    quotation_marks,
    rails_migrations_pending,
    react_native_command_unrecognized,
    remove_shell_prompt_literal,
    remove_trailing_cedilla,
    rm_dir,
    rm_root,
    scm_correction,
    sed_unterminated_s,
    sl_ls,
    ssh_known_hosts,
    sudo,
    sudo_command_from_user_path,
    switch_lang,
    systemctl,
    terraform_init,
    terraform_no_command,
    test_py,
    tmux,
    touch,
    tsuru_login,
    tsuru_not_command,
    unknown_command,
    unsudo,
    vagrant_up,
    whois,
    workon_doesnt_exists,
    wrong_hyphen_before_subcommand,
    yarn_alias,
    yarn_command_not_found,
    yarn_command_replaced,
    yarn_help,
    yum_invalid_operation,
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::types::Command;

    /// Calls of rules recorded while running the test suite of the original
    /// Python implementation, with the results it gave.
    const PYTHON_CASES: &str = include_str!("testdata/python_cases.json");

    #[test]
    fn rules_behave_like_in_python() {
        let cases: Vec<Value> = serde_json::from_str(PYTHON_CASES).unwrap();
        let rules = super::all();
        let mut failures = Vec::new();

        for case in &cases {
            let name = case["rule"].as_str().unwrap();
            let rule = rules
                .iter()
                .find(|rule| rule.name == name)
                .unwrap_or_else(|| panic!("no rule {}", name));
            let command = Command::new(case["script"].as_str().unwrap(), case["output"].as_str());
            let (expected, actual) = if case["kind"] == "match" {
                (case["result"].clone(), Value::from(rule.is_match(&command)))
            } else {
                let scripts: Vec<String> = rule
                    .get_corrected_commands(&command)
                    .into_iter()
                    .map(|corrected| corrected.script)
                    .collect();
                (case["result"].clone(), Value::from(scripts))
            };
            if expected != actual {
                failures.push(format!(
                    "{} {}\n  script:   {:?}\n  output:   {:?}\n  expected: {}\n  actual:   {}",
                    name, case["kind"], command.script, command.output, expected, actual
                ));
            }
        }

        assert!(
            failures.is_empty(),
            "{} of {} cases failed:\n{}",
            failures.len(),
            cases.len(),
            failures.join("\n")
        );
    }
}
