use serde::{Deserialize, Serialize};

use super::imageset::ImageSet;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Artist {
    pub image: ImageSet,
    pub name: String,
    pub url: String,
}
