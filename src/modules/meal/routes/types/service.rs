use super::types::response;
use crate::modules::kitchen::types;

pub fn service() -> response::Response {
    Ok(response::Success::Types(
        Vec::from(&types::KITCHEN_TYPES)
            .into_iter()
            .map(String::from)
            .collect(),
    ))
}
