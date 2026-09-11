use vigil_model::StoreStatus;

pub(super) fn limitations(store: Option<&StoreStatus>) -> Vec<String> {
    let mut said = vec![
        "Nothing is buffered for sending: a finding is delivered as it is found, and a receiver \
         that is down misses that delivery rather than being caught up later."
            .to_string(),
    ];
    if store.is_none() {
        said.push(
            "This build does not say what its local history holds. It is on disk under the state \
             directory, and `jq` reads it."
                .to_string(),
        );
    }
    said
}
