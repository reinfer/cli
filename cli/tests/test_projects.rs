use crate::common::TestCli;
use pretty_assertions::assert_eq;
use reinfer_client::Project;
use uuid::Uuid;

pub struct TestProject {
    name: String,
}

impl TestProject {
    pub fn new() -> Self {
        TestProject::new_args(&[])
    }

    pub fn new_args(args: &[&str]) -> Self {
        let cli = TestCli::get();
        let name = Uuid::new_v4().to_string();

        let user = cli.user();

        let output = cli.run(
            ["create", "project", &name, "--user-ids", &user.id.0]
                .iter()
                .chain(args),
        );

        // Creating projects is complex and can sometimes cause race conditions, sleeping after
        // creating should hopefully reduce the probability of that.
        std::thread::sleep(std::time::Duration::from_millis(200));

        assert!(output.contains(&name));

        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let output = TestCli::get().run(&["delete", "project", self.name(), "--force"]);
        assert!(output.is_empty());
    }
}

#[test]
fn test_test_project() {
    let cli = TestCli::get();
    let project = TestProject::new();

    let name = project.name().to_owned();

    let output = cli.run(&["get", "projects"]);
    assert!(output.contains(&name));

    drop(project);

    // RAII TestProject; should automatically clean up the temporary project on drop.
    let output = cli.run(&["get", "projects"]);
    assert!(!output.contains(&name));
}

#[test]
fn test_list_multiple_projects() {
    let cli = TestCli::get();
    let project1 = TestProject::new();
    let project2 = TestProject::new();

    let output = cli.run(&["get", "projects"]);
    assert!(output.contains(project1.name()));
    assert!(output.contains(project2.name()));

    let output = cli.run(&["get", "projects", project1.name()]);
    assert!(output.contains(project1.name()));
    assert!(!output.contains(project2.name()));

    let output = cli.run(&["get", "projects", project2.name()]);
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
        let output = cli.run(&["--output=json", "get", "projects", project.name()]);
        serde_json::from_str::<Project>(&output).unwrap().into()
    };

    let mut expected_project_info = ProjectInfo {
        name: project.name().to_owned(),
        title: "some title".to_owned(),
        description: "some description".to_owned(),
    };
    assert_eq!(get_project_info(), expected_project_info);

    // An empty update should be fine
    cli.run(&["update", "project", project.name()]);
    assert_eq!(get_project_info(), expected_project_info);

    // Partial update
    cli.run(&["update", "project", "--title=updated title", project.name()]);
    expected_project_info.title = "updated title".to_owned();
    assert_eq!(get_project_info(), expected_project_info);

    // Should be able to update all fields
    cli.run(&[
        "update",
        "project",
        "--title=updated title for second time",
        "--description=updated description",
        project.name(),
    ]);
    expected_project_info.title = "updated title for second time".to_owned();
    expected_project_info.description = "updated description".to_owned();
    assert_eq!(get_project_info(), expected_project_info);
}

#[test]
fn test_project_force_delete() {
    let cli = TestCli::get();
    let project = TestProject::new();

    let name = project.name().to_owned();
    let source_name = format!("{}/a-source", name);

    cli.run(&["create", "source", &source_name]);

    // Regular delete fails because of the source

    let output = cli
        .command()
        .args(&["delete", "project", &name])
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

    let output = cli.run(&["get", "projects"]);
    assert!(output.contains(&name));

    // Force delete succeeds

    cli.run(&["delete", "project", &name, "--force"]);

    // The project no longer exists

    let output = cli.run(&["get", "projects"]);
    assert!(!output.contains(&name));

    // To avoid panic on drop because the project has already been deleted.
    std::mem::forget(project);
}
