mod content_type;
mod responses;
mod type_flyweight;

pub use crate::responses::{build_response, get_request_details};
pub use crate::type_flyweight::BoxedResponse;
