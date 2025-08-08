pub fn stop_container(id: &str) {
    std::process::Command::new("docker")
        .args(["rm", "-f", id])
        .output()
        .unwrap();
}

pub fn start_container(command: &str, args: &[&str], is_ready: impl Fn(&str) -> bool) -> String {
    let r = std::process::Command::new(command)
        .args(args)
        .output()
        .unwrap();
    assert!(r.status.success());
    let id = String::from_utf8(r.stdout).unwrap();

    for _ in 0..20 {
        if is_ready(&id) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    id
}
