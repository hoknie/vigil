use std::fs;
use std::path::PathBuf;

use super::*;

const ID: &str = "3ab1c0f2d4e5a6b7c8d9e0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5";

const OTHER_ID: &str = "9f2e8d7c6b5a4039281706f5e4d3c2b1a0998877665544332211ffeeddccbbaa";

const THIS_HOST: &str = "\
25 30 254:1 / / rw,relatime shared:1 - ext4 /dev/vda1 rw\n\
27 25 0:26 / /run rw,nosuid,nodev shared:5 - tmpfs tmpfs rw,size=802016k,mode=755\n";

const OF_A_CONTAINER: &str = "\
447 446 0:98 / / rw,relatime - overlay overlay rw\n\
452 447 254:1 /srv/www /usr/share/nginx/html ro,relatime - ext4 /dev/vda1 ro\n\
453 447 0:26 /docker.sock /var/run/docker.sock rw,relatime - tmpfs tmpfs rw\n";

const UNPRIVILEGED: &str = "Name:\tnginx\nCapEff:\t00000000a80425fb\n";

const PRIVILEGED: &str = "Name:\tsh\nCapEff:\t000001ffffffffff\n";

struct Bench {
    directory: PathBuf,
}

impl Bench {
    fn new(named: &str) -> Bench {
        let directory = std::env::temp_dir().join(format!(
            "vigil-containers-{named}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(directory.join("self")).expect("a bench to read from");
        let bench = Bench { directory };
        bench.write("self/mountinfo", THIS_HOST);
        bench.of_this_host(1);
        bench
    }

    fn write(&self, under: &str, text: &str) {
        fs::write(self.directory.join(under), text).expect("writes");
    }

    fn process(&self, pid: u32) -> PathBuf {
        let at = self.directory.join(pid.to_string());
        fs::create_dir_all(&at).expect("a process to read");
        at
    }

    fn of_this_host(&self, pid: u32) {
        self.process(pid);
        self.write(&format!("{pid}/cgroup"), "0::/system.slice/nginx.service\n");
        self.write(&format!("{pid}/status"), UNPRIVILEGED);
        self.write(&format!("{pid}/mountinfo"), THIS_HOST);
    }

    fn in_a_container(&self, pid: u32, id: &str, status: &str) {
        self.process(pid);
        self.write(
            &format!("{pid}/cgroup"),
            &format!("0::/system.slice/docker-{id}.scope\n"),
        );
        self.write(&format!("{pid}/status"), status);
        self.write(&format!("{pid}/mountinfo"), OF_A_CONTAINER);
    }

    fn socket(&self, mode: u32) -> String {
        let at = self.directory.join("docker.sock");
        fs::write(&at, "").expect("writes");
        fs::set_permissions(&at, std::os::unix::fs::PermissionsExt::from_mode(mode))
            .expect("sets the mode");
        at.display().to_string()
    }

    fn collector(&self, sockets: &[&str]) -> ContainersCollector {
        ContainersCollector::with_sources(
            || "2026-09-11T12:00:00.000Z".to_string(),
            &self.directory,
            sockets,
        )
    }
}

impl Drop for Bench {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn said(health: &Health) -> String {
    match health {
        Health::Ok => String::new(),
        Health::Degraded(detail) | Health::Unavailable(detail) => detail.clone(),
    }
}

#[test]
fn a_container_is_one_row_however_many_processes_it_runs_and_this_host_itself_is_none() {
    let bench = Bench::new("one-row");
    bench.in_a_container(1041, ID, UNPRIVILEGED);
    bench.in_a_container(1042, ID, UNPRIVILEGED);
    bench.in_a_container(2077, OTHER_ID, PRIVILEGED);
    let collector = bench.collector(&[]);

    let reading = collector.collect().expect("reads");
    let containers: Vec<&String> = reading
        .items
        .keys()
        .filter(|key| key.starts_with("container|"))
        .collect();

    assert_eq!(collector.available(), Health::Ok);
    assert_eq!(containers.len(), 2, "{containers:?}");
    assert_eq!(
        reading.items[&format!("container|{}", &ID[..12])]["id"],
        ID,
        "the row is named by the twelve characters a person reads, and carries the whole one"
    );
}

#[test]
fn a_path_of_this_host_bound_into_a_container_is_named_as_this_host_names_it() {
    let bench = Bench::new("mounts");
    bench.in_a_container(1041, ID, UNPRIVILEGED);

    let reading = bench.collector(&[]).collect().expect("reads");
    let row = &reading.items[&format!("container|{}", &ID[..12])];

    assert_eq!(row["host_paths"][0], "/run/docker.sock");
    assert_eq!(row["host_paths"][1], "/srv/www");
    assert_eq!(row["mounts_readable"], true);
    assert_eq!(row["capabilities_effective"], "00000000a80425fb");
}

#[test]
fn a_container_whose_capabilities_could_not_be_read_is_a_row_and_a_complaint_and_not_a_silence() {
    let bench = Bench::new("unread");
    bench.in_a_container(1041, ID, UNPRIVILEGED);
    fs::remove_file(bench.directory.join("1041/status")).expect("removes");
    fs::remove_file(bench.directory.join("1041/mountinfo")).expect("removes");
    let collector = bench.collector(&[]);

    let reading = collector.collect().expect("reads");
    let row = &reading.items[&format!("container|{}", &ID[..12])];
    let health = collector.available();

    assert_eq!(row["capabilities_effective"], serde_json::Value::Null);
    assert_eq!(row["mounts_readable"], false);
    assert!(matches!(health, Health::Degraded(_)), "{health:?}");
    assert!(
        said(&health).contains("privileged container among them would not be seen"),
        "{}",
        said(&health)
    );
}

#[test]
fn the_socket_a_runtime_listens_on_is_read_with_the_mode_that_says_who_may_reach_it() {
    let bench = Bench::new("socket");
    let path = bench.socket(0o666);

    let reading = bench.collector(&[&path]).collect().expect("reads");
    let row = &reading.items[&format!("container-socket|{path}")];

    assert_eq!(row["mode"], "0666");
    assert_eq!(row["path"], path);
}

#[test]
fn a_socket_that_is_not_there_is_no_row_and_a_host_with_no_runtime_is_not_a_refusal() {
    let bench = Bench::new("no-socket");

    let reading = bench
        .collector(&["/there/is/no/such/socket"])
        .collect()
        .expect("reads");

    assert!(
        reading.items.is_empty(),
        "a host that runs no containers and no runtime is a host, and this is what it reads \
         like: {:?}",
        reading.items
    );
}

#[test]
fn a_proc_this_agent_cannot_list_is_never_a_host_running_no_containers() {
    let bench = Bench::new("no-proc");
    let collector = ContainersCollector::with_sources(
        || "2026-09-11T12:00:00.000Z".to_string(),
        bench.directory.join("not-a-proc"),
        &[],
    );

    let health = collector.available();

    assert!(matches!(health, Health::Unavailable(_)), "{health:?}");
    assert!(matches!(collector.collect(), Err(CollectError::Absent(_))));
}
