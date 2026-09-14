use vigil_view::Facet;

pub(in crate::ui::app) enum Narrowing {
    Everything,
    Only(Facet),
    Any(&'static str),
}

impl Narrowing {
    pub(super) fn said(&self) -> String {
        match self {
            Narrowing::Everything => "everything".to_string(),
            Narrowing::Only(facet) => format!("only {} {}", facet.name, facet.value),
            Narrowing::Any(name) => format!("any {name}"),
        }
    }
}
