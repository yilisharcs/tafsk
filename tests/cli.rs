use std::error::Error;
use std::fs;

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn should_fail_when_no_task_store_is_found() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = tempfile::tempdir()?;

        let mut cmd = cargo_bin_cmd!("tafsk");

        cmd.current_dir(tmp.path()) // Sets the current directory to /tmp/.tmpXXXXXX
                .env_remove("TAFSK_STORE_DIR")
                .arg("add")
                .arg("Should Fail");

        cmd.assert()
                .failure()
                .stderr(predicate::str::contains("No task store found"));

        Ok(())
}

#[test]
fn should_create_task_in_global_store_when_env_var_is_set() -> Result<(), Box<dyn Error>> {
        let tmp = tempfile::tempdir()?;
        let global_store = tmp.path().join("global_tasks");
        let _ = fs::create_dir(&global_store);

        let mut cmd = cargo_bin_cmd!("tafsk");
        cmd.current_dir(tmp.path())
                .env("TAFSK_STORE_DIR", &global_store)
                .arg("add")
                .arg("Global Task");

        cmd.assert().success();

        let entries = fs::read_dir(&global_store)?;
        assert!(entries.count() > 0);

        Ok(())
}

#[test]
fn should_create_local_store_when_init_is_run() -> Result<(), Box<dyn Error>> {
        let tmp = tempfile::tempdir()?;
        // Create a mock global store location
        let mock_global_store = tmp.path().join("mock_global_store");
        fs::create_dir(&mock_global_store)?; // It must exist for the env var to be valid in `Store::new`

        let mut cmd = cargo_bin_cmd!("tafsk");

        cmd.current_dir(tmp.path())
                .env("TAFSK_STORE_DIR", &mock_global_store) // Set global env var
                .arg("init");

        cmd.assert().success();

        let local_dir = tmp.path().join("tasks");
        assert!(
                local_dir.exists(),
                "Local 'tasks' directory should be created"
        );

        let entries = fs::read_dir(&local_dir)?;
        assert!(
                entries.count() > 0,
                "Local 'tasks' directory should not be empty"
        );

        let local_conf = local_dir.join(".config");
        assert!(
                local_conf.exists(),
                "Local '.config' file should be created"
        );

        let mock_global_tasks_dir = mock_global_store.join("tasks");
        assert!(
                !mock_global_tasks_dir.exists(),
                "Init should NOT create tasks in the global store path"
        );

        Ok(())
}
