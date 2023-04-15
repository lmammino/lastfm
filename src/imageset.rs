use std::collections::HashMap;

use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageSet {
    pub small: Option<String>,
    pub medium: Option<String>,
    pub large: Option<String>,
    pub extralarge: Option<String>,
}

impl<'de> Deserialize<'de> for ImageSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_images: Vec<HashMap<String, String>> = Deserialize::deserialize(deserializer)?;

        let mut image_set = ImageSet {
            small: None,
            medium: None,
            large: None,
            extralarge: None,
        };

        for image in raw_images {
            let size = image
                .get("size")
                .ok_or_else(|| D::Error::missing_field("size"))?;
            let url = image
                .get("#text")
                .ok_or_else(|| D::Error::missing_field("#text"))?;
            match size.as_str() {
                "small" => image_set.small = Some(url.to_string()),
                "medium" => image_set.medium = Some(url.to_string()),
                "large" => image_set.large = Some(url.to_string()),
                "extralarge" => image_set.extralarge = Some(url.to_string()),
                _ => (),
            }
        }

        Ok(image_set)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn it_deserializes_correctly() {
        let json_value = json!([
          {
            "#text": "https://url1.com",
            "size": "small"
          }
        ]);

        let image_set: ImageSet = serde_json::from_value(json_value).unwrap();

        assert_eq!(image_set.small, Some("https://url1.com".to_string()));
        assert_eq!(image_set.medium, None);
        assert_eq!(image_set.large, None);
        assert_eq!(image_set.extralarge, None);
    }

    #[test]
    fn it_deserializes_everything_correctly() {
        let json_value = json!([
          {
            "#text": "https://url1.com",
            "size": "small"
          },
          {
            "#text": "https://url2.com",
            "size": "medium"
          },
          {
            "#text": "https://url3.com",
            "size": "large"
          },
          {
            "#text": "https://url4.com",
            "size": "extralarge"
          }
        ]);

        let image_set: ImageSet = serde_json::from_value(json_value).unwrap();

        assert_eq!(image_set.small, Some("https://url1.com".to_string()));
        assert_eq!(image_set.medium, Some("https://url2.com".to_string()));
        assert_eq!(image_set.large, Some("https://url3.com".to_string()));
        assert_eq!(image_set.extralarge, Some("https://url4.com".to_string()));
    }

    #[test]
    fn it_fails_with_missing_fields() {
        let json_value = json!([
          {
            "size": "small"
          }
        ]);

        let image_set: Result<ImageSet, _> = serde_json::from_value(json_value);
        assert_eq!("missing field `#text`", image_set.unwrap_err().to_string());

        let json_value = json!([
          {
            "#text": "blah"
          }
        ]);

        let image_set: Result<ImageSet, _> = serde_json::from_value(json_value);
        assert_eq!("missing field `size`", image_set.unwrap_err().to_string());
    }
}
