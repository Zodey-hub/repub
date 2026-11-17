use tiny_database::{NoIndex, Record, rkyv};

/// The image types a cover can have: a browser only shows these as plain pictures, an SVG could run a script.
const COVER_MIME_TYPES: [&str; 4] = ["image/jpeg", "image/png", "image/gif", "image/webp"];

/// The cover of a book: the image from its EPUB file, in its own table so that listing the books does not read the images.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = tiny_database::rkyv)]
pub struct Cover {
    id: u64,
    mime_type: String,
    image: Vec<u8>,
}

impl Cover {
    /// The cover of the book `id`, or `None` if `mime_type` is not a JPEG, PNG, GIF or WebP.
    pub fn new(id: u64, image: Vec<u8>, mime_type: &str) -> Option<Self> {
        let mime_type = mime_type.to_ascii_lowercase();
        let is_allowed = COVER_MIME_TYPES.contains(&mime_type.as_str());

        is_allowed.then_some(Self {
            id,
            mime_type,
            image,
        })
    }

    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    /// The image of the cover, in the format of its MIME type.
    pub fn into_image(self) -> Vec<u8> {
        self.image
    }
}

impl Record for Cover {
    const TABLE: &'static str = "covers";
    type Index = NoIndex;

    fn id(&self) -> u64 {
        self.id
    }

    fn index_keys(&self) -> Vec<(NoIndex, Vec<u8>)> {
        Vec::new()
    }
}
