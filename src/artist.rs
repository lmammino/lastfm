//! # Artist
//!
//! defines the [`Artist`] struct and its methods.
use crate::imageset::ImageSet;
use serde::{Deserialize, Serialize};

/// A Last.fm artist.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Artist {
    pub image: ImageSet,
    pub name: String,
    pub url: String,
}
