use config::Config;

use crate::type_flyweight::{AvailableEncodings, ServiceRequirements};

pub fn get_service_requirements(config: &Config) -> ServiceRequirements {
    ServiceRequirements {
        directory: config.directory.clone(),
        encodings: AvailableEncodings::new(&config.content_encodings),
        filepath_404s: config.filepath_404s.clone(),
    }
}
