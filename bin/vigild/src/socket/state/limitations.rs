use vigil_model::{BufferStatus, StoreStatus};

pub(super) fn limitations(
    store: Option<&StoreStatus>,
    buffers: Option<&[BufferStatus]>,
) -> Vec<String> {
    let mut said = Vec::new();
    if buffers.is_none() {
        said.push(
            "This build does not say how much is waiting for a receiver that would not take it. \
             It is on disk under the state directory, and `jq` reads it."
                .to_string(),
        );
    }
    if store.is_none() {
        said.push(
            "This build does not say what its local history holds. It is on disk under the state \
             directory, and `jq` reads it."
                .to_string(),
        );
    }
    said
}
