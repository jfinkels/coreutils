use crate::common::util::*;

#[test]
fn test_realpath_current_directory() {
    let (at, mut ucmd) = at_and_ucmd!();
    let expect = at.root_dir_resolved() + "\n";
    ucmd.arg(".").succeeds().stdout_is(expect);
}

#[test]
fn test_realpath_long_redirection_to_current_dir() {
    let (at, mut ucmd) = at_and_ucmd!();
    // Create a 256-character path to current directory
    let dir = path_concat!(".", ..128);
    let expect = at.root_dir_resolved() + "\n";
    ucmd.arg(dir).succeeds().stdout_is(expect);
}

#[test]
fn test_realpath_long_redirection_to_root() {
    // Create a 255-character path to root
    let dir = path_concat!("..", ..85);
    let expect = get_root_path().to_owned() + "\n";
    new_ucmd!().arg(dir).succeeds().stdout_is(expect);
}

#[test]
fn test_realpath_file_and_links() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;

    at.touch("foo");
    at.symlink_file("foo", "bar");

    scene.ucmd().arg("foo").succeeds().stdout_contains("foo\n");
    scene.ucmd().arg("bar").succeeds().stdout_contains("foo\n");
}

#[test]
fn test_realpath_file_and_links_zero() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;

    at.touch("foo");
    at.symlink_file("foo", "bar");

    scene
        .ucmd()
        .arg("foo")
        .arg("-z")
        .succeeds()
        .stdout_contains("foo\u{0}");

    scene
        .ucmd()
        .arg("bar")
        .arg("-z")
        .succeeds()
        .stdout_contains("foo\u{0}");
}

#[test]
fn test_realpath_file_and_links_strip() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;

    at.touch("foo");
    at.symlink_file("foo", "bar");

    scene
        .ucmd()
        .arg("foo")
        .arg("-s")
        .succeeds()
        .stdout_contains("foo\n");

    scene
        .ucmd()
        .arg("bar")
        .arg("-s")
        .succeeds()
        .stdout_contains("bar\n");
}

#[test]
fn test_realpath_file_and_links_strip_zero() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;

    at.touch("foo");
    at.symlink_file("foo", "bar");

    scene
        .ucmd()
        .arg("foo")
        .arg("-s")
        .arg("-z")
        .succeeds()
        .stdout_contains("foo\u{0}");

    scene
        .ucmd()
        .arg("bar")
        .arg("-s")
        .arg("-z")
        .succeeds()
        .stdout_contains("bar\u{0}");
}

#[test]
fn test_logical() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;

    at.mkdir("dir1");
    at.mkdir("dir1/dir2");
    at.symlink_dir("dir1/dir2", "link_to_dir2");

    let expected = format!("{}\n", at.as_string());
    scene
        .ucmd()
        .args(&["-L", "link_to_dir2/.."])
        .succeeds()
        .stdout_is(expected);
}

#[test]
fn test_physical() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;

    at.mkdir("dir1");
    at.mkdir("dir1/dir2");
    at.symlink_dir("dir1/dir2", "link_to_dir2");

    let expected = format!("{}/dir1\n", at.as_string());
    scene
        .ucmd()
        .args(&["-P", "link_to_dir2/.."])
        .succeeds()
        .stdout_is(expected);
}

#[test]
fn test_physical_default() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;

    at.mkdir("dir1");
    at.mkdir("dir1/dir2");
    at.symlink_dir("dir1/dir2", "link_to_dir2");

    let expected = format!("{}/dir1\n", at.as_string());
    scene
        .ucmd()
        .arg("link_to_dir2/..")
        .succeeds()
        .stdout_is(expected);
}

#[test]
fn test_physical_overrides_logical() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;

    at.mkdir("dir1");
    at.mkdir("dir1/dir2");
    at.symlink_dir("dir1/dir2", "link_to_dir2");

    let expected = format!("{}/dir1\n", at.as_string());
    scene
        .ucmd()
        .args(&["-L", "-P", "link_to_dir2/.."])
        .succeeds()
        .stdout_is(expected);
}

#[test]
fn test_logical_overrides_physical() {
    let scene = TestScenario::new(util_name!());
    let at = &scene.fixtures;

    at.mkdir("dir1");
    at.mkdir("dir1/dir2");
    at.symlink_dir("dir1/dir2", "link_to_dir2");

    let expected = format!("{}\n", at.as_string());
    scene
        .ucmd()
        .args(&["-P", "-L", "link_to_dir2/.."])
        .succeeds()
        .stdout_is(expected);
}
