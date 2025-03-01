mod available_encodings;
mod content_type;
mod get_range_response;
mod get_response;
mod head_response;
mod last_resort_response;
mod response_paths;
mod responses;
mod type_flyweight;

pub use crate::responses::build_response;
pub use crate::type_flyweight::BoxedResponse;
