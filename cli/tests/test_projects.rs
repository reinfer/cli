use crate::common::TestCli;
use pretty_assertions::assert_eq;
use reinfer_client::{Project, User};
use uuid::Uuid;

pub struct TestProject {
    name: String,
}

impl TestProject {
    pub fn new() -> Self {
        TestProject::new_args(&[])
    }

    fn wait_for_project(cli: &TestCli, name: &str, user: &User) -> bool {
        // Creating projects is complex and can sometimes cause race conditions,
        // So we loop until the project is created with our user in, or we time out.
        let start_time = std::time::Instant::now();
        loop {
            let output = cli.run(["get", "users", "--project", name, "--user", &user.id.0]);

            if output.contains(&user.id.0) {
                return true;
            }

            if start_time.elapsed().as_secs() > 30 {
                log::warn!("Timed out waiting for project to be created");
                return false;
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    pub fn new_args(args: &[&str]) -> Self {
        let cli = TestCli::get();

        // We retry up to 5 times due to some flakey race conditions
        for _ in 0..5 {
            let name = Uuid::new_v4().to_string();

            let user = cli.user();

            cli.run(
                ["create", "project", &name, "--user-ids", &user.id.0]
                    .iter()
                    .chain(args),
            );

            if Self::wait_for_project(cli, &name, &user) {
                return Self { name };
            }
        }
        panic!("Could not create project: Timed out waiting for project to be created");
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let output = TestCli::get().run(["delete", "project", self.name(), "--force"]);
        assert!(output.is_empty());
    }
}

#[test]
fn test_test_project() {
    let cli = TestCli::get();
    let project = TestProject::new();

    let name = project.name().to_owned();

    let output = cli.run(["get", "projects"]);
    assert!(output.contains(&name));

    drop(project);

    // RAII TestProject; should automatically clean up the temporary project on drop.
    let output = cli.run(["get", "projects"]);
    assert!(!output.contains(&name));
}

#[test]
fn test_list_multiple_projects() {
    let cli = TestCli::get();
    let project1 = TestProject::new();
    let project2 = TestProject::new();

    let output = cli.run(["get", "projects"]);
    assert!(output.contains(project1.name()));
    assert!(output.contains(project2.name()));

    let output = cli.run(["get", "projects", project1.name()]);
    assert!(output.contains(project1.name()));
    assert!(!output.contains(project2.name()));

    let output = cli.run(["get", "projects", project2.name()]);
    assert!(!output.contains(project1.name()));
    assert!(output.contains(project2.name()));
}

#[test]
fn test_create_update_project_custom() {
    let cli = TestCli::get();
    let project = TestProject::new_args(&["--title=some title", "--description=some description"]);

    /// A subset of project fields that we can easily check for equality accross
    #[derive(PartialEq, Eq, Debug)]
    struct ProjectInfo {
        name: String,
        title: String,
        description: String,
    }

    impl From<Project> for ProjectInfo {
        fn from(project: Project) -> ProjectInfo {
            ProjectInfo {
                name: project.name.0,
                title: project.title,
                description: project.description,
            }
        }
    }

    let get_project_info = || -> ProjectInfo {
        let output = cli.run(["--output=json", "get", "projects", project.name()]);
        serde_json::from_str::<Project>(&output).unwrap().into()
    };

    let mut expected_project_info = ProjectInfo {
        name: project.name().to_owned(),
        title: "some title".to_owned(),
        description: "some description".to_owned(),
    };
    assert_eq!(get_project_info(), expected_project_info);

    // An empty update should be fine
    cli.run(["update", "project", project.name()]);
    assert_eq!(get_project_info(), expected_project_info);

    // Partial update
    cli.run(["update", "project", "--title=updated title", project.name()]);
    "updated title".clone_into(&mut expected_project_info.title);
    assert_eq!(get_project_info(), expected_project_info);

    // Should be able to update all fields
    cli.run([
        "update",
        "project",
        "--title=updated title for second time",
        "--description=updated description",
        project.name(),
    ]);
    "updated title for second time".clone_into(&mut expected_project_info.title);
    "updated description".clone_into(&mut expected_project_info.description);
    assert_eq!(get_project_info(), expected_project_info);
}

#[test]
fn test_project_force_delete() {
    let cli = TestCli::get();
    let project = TestProject::new();

    let name = project.name().to_owned();
    let source_name = format!("{name}/a-source");

    cli.run(["create", "source", &source_name]);

    // Regular delete fails because of the source

    let output = cli
        .command()
        .args(["delete", "project", &name])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("API request failed with 409 Conflict: Project contains child resources but force deletion was not requested: {\"sources\": 1}"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The project still exists

    let output = cli.run(["get", "projects"]);
    assert!(output.contains(&name));

    // Force delete succeeds

    cli.run(["delete", "project", &name, "--force"]);

    // The project no longer exists

    let output = cli.run(["get", "projects"]);
    assert!(!output.contains(&name));

    // To avoid panic on drop because the project has already been deleted.
    std::mem::forget(project);
}
