use crate::providers::ProviderKind;

#[derive(Getters, new, PartialEq)]
pub struct SpecifiedTarget {
    provider_kind: Option<ProviderKind>,
    name: String,
}
