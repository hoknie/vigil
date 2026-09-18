use crate::types::{ANSWERED, Answer, Dump, Engine, Subject};

pub const AT: &str = "2026-09-17T09:00:01.000Z";

pub const PROGRAM: &str = "/usr/bin/docker";

pub const DEADLINE_SECONDS: u64 = 10;

pub const DAEMON_JSON: &str = r#"{
  "insecure-registries": ["registry.local:5000"],
  "registry-mirrors": ["https://mirror.example.com"],
  "log-driver": "json-file"
}
"#;

const IMAGES: &str = concat!(
    r#"{"Containers":"2","CreatedAt":"2026-08-30 11:04:12 +0000 UTC","CreatedSince":"2 weeks ago","Digest":"sha256:9a7f6e5d4c3b2a190807f6e5d4c3b2a1908070605040302010f0e0d0c0b0a090","ID":"18ad9bdc4c87","Repository":"nginx","SharedSize":"N/A","Size":"78.5MB","Tag":"1.27-alpine","UniqueSize":"N/A"}"#,
    "\n",
    r#"{"Containers":"1","CreatedAt":"2026-09-14 08:21:55 +0000 UTC","CreatedSince":"3 days ago","Digest":"<none>","ID":"c4b2e1f09a33","Repository":"shop/api","SharedSize":"N/A","Size":"412MB","Tag":"2026.09.1","UniqueSize":"N/A"}"#,
    "\n",
    r#"{"Containers":"1","CreatedAt":"2026-09-16 19:02:41 +0000 UTC","CreatedSince":"20 hours ago","Digest":"<none>","ID":"5f0c1ad8b29e","Repository":"<none>","SharedSize":"N/A","Size":"1.02GB","Tag":"<none>","UniqueSize":"N/A"}"#,
    "\n",
);

const VOLUMES: &str = concat!(
    r#"{"Availability":"N/A","Driver":"local","Group":"N/A","Labels":"com.docker.compose.project=shop,com.docker.compose.volume=database","Links":"1","Mountpoint":"/var/lib/docker/volumes/shop_database/_data","Name":"shop_database","Scope":"local","Size":"N/A","Status":"N/A"}"#,
    "\n",
    r#"{"Availability":"N/A","Driver":"local","Group":"N/A","Labels":"com.docker.volume.anonymous=","Links":"0","Mountpoint":"/var/lib/docker/volumes/2e10970a75cc/_data","Name":"2e10970a75cc","Scope":"local","Size":"N/A","Status":"N/A"}"#,
    "\n",
);

const NETWORKS: &str = concat!(
    r#"{"CreatedAt":"2024-11-11 09:59:19 +0000 UTC","Driver":"bridge","ID":"bd8aceefd293","IPv4":"true","IPv6":"false","Internal":"false","Labels":"","Name":"bridge","Scope":"local"}"#,
    "\n",
    r#"{"CreatedAt":"2024-11-11 09:59:19 +0000 UTC","Driver":"host","ID":"b60b79f81273","IPv4":"true","IPv6":"false","Internal":"false","Labels":"","Name":"host","Scope":"local"}"#,
    "\n",
    r#"{"CreatedAt":"2026-09-01 07:14:02 +0000 UTC","Driver":"bridge","ID":"71c05aa4e8d1","IPv4":"true","IPv6":"false","Internal":"false","Labels":"com.docker.compose.network=default,com.docker.compose.project=shop","Name":"shop_default","Scope":"local"}"#,
    "\n",
);

const CONTAINERS: &str = concat!(
    r#"{"Command":"\"nginx -g 'daemon of…\"","CreatedAt":"2026-09-01 07:14:04 +0000 UTC","ID":"0f72b16f7cc6","Image":"nginx:1.27-alpine","Labels":"com.docker.compose.project=shop,com.docker.compose.project.config_files=/srv/shop/compose.yaml,com.docker.compose.project.working_dir=/srv/shop,com.docker.compose.service=web,com.shop.api-token=s.9a7f6e5d4c3b","LocalVolumes":"0","Mounts":"/etc,shop_database","Names":"shop-web-1","Networks":"shop_default","Ports":"0.0.0.0:443->443/tcp","RunningFor":"16 days ago","Size":"0B","State":"running","Status":"Up 16 days"}"#,
    "\n",
    r#"{"Command":"\"/srv/api serve\"","CreatedAt":"2026-09-14 08:22:10 +0000 UTC","ID":"3b6d9e0a1f52","Image":"shop/api:2026.09.1","Labels":"com.docker.compose.project=shop,com.docker.compose.project.config_files=/srv/shop/compose.yaml,com.docker.compose.project.working_dir=/srv/shop,com.docker.compose.service=api","LocalVolumes":"1","Mounts":"shop_database","Names":"shop-api-1","Networks":"shop_default","Ports":"","RunningFor":"3 days ago","Size":"0B","State":"running","Status":"Up 3 days"}"#,
    "\n",
    r#"{"Command":"\"/agent --collect\"","CreatedAt":"2026-09-16 19:03:02 +0000 UTC","ID":"a1d4c7b0e935","Image":"5f0c1ad8b29e","Labels":"com.docker.compose.project=metrics,com.docker.compose.project.config_files=/srv/metrics/compose.yaml,com.docker.compose.project.working_dir=/srv/metrics,com.docker.compose.service=agent","LocalVolumes":"0","Mounts":"/proc,/sys","Names":"metrics-agent-1","Networks":"host","Ports":"","RunningFor":"20 hours ago","Size":"0B","State":"running","Status":"Up 20 hours"}"#,
    "\n",
);

const INFORMATION: &str = r#"{"ID":"3397979a-9425-4a59-b64d-8f692526990c","Containers":3,"ContainersRunning":3,"Images":3,"Driver":"overlay2","CgroupDriver":"systemd","CgroupVersion":"2","LoggingDriver":"json-file","SecurityOptions":["name=seccomp,profile=builtin","name=cgroupns"],"ServerVersion":"27.1.1","DockerRootDir":"/var/lib/docker","SystemTime":"2026-09-17T09:00:01.412339Z","KernelVersion":"6.1.0-23-amd64","OperatingSystem":"Debian GNU/Linux 12 (bookworm)","OSType":"linux","Architecture":"x86_64","NCPU":4,"MemTotal":8232456192}"#;

pub fn dump() -> Dump {
    let mut dump = Dump::present(Engine::Docker.name(), AT, DEADLINE_SECONDS, PROGRAM);

    for (subject, printed, milliseconds) in [
        (Subject::Image, IMAGES, 61),
        (Subject::Volume, VOLUMES, 47),
        (Subject::Network, NETWORKS, 44),
        (Subject::Container, CONTAINERS, 58),
        (Subject::Engine, INFORMATION, 72),
    ] {
        dump.asked.insert(
            subject.as_str().to_string(),
            answered(Engine::Docker, subject, printed, milliseconds),
        );
    }

    dump
}

pub fn answered(engine: Engine, subject: Subject, printed: &str, milliseconds: u64) -> Answer {
    let arguments = engine
        .asks()
        .iter()
        .find(|asked| asked.subject == subject)
        .map(|asked| {
            asked
                .arguments
                .iter()
                .map(|word| (*word).to_string())
                .collect()
        })
        .unwrap_or_default();

    Answer {
        state: ANSWERED.to_string(),
        arguments,
        milliseconds,
        printed: printed.to_string(),
        truncated: false,
        status: Some(0),
        why: None,
    }
}
