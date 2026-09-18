use vigil_module::Module;

pub fn modules() -> Vec<Box<dyn Module>> {
    vec![
        Box::new(vigil_network::Network),
        Box::new(vigil_users::Users),
        Box::new(vigil_processes::Processes),
        Box::new(vigil_launches::Launches),
        Box::new(vigil_persistence::Persistence),
        Box::new(vigil_firewall::Firewall),
        Box::new(vigil_resources::Resources),
        Box::new(vigil_files::Files),
        Box::new(vigil_containers::Containers),
        Box::new(vigil_engines::Engines),
    ]
}
