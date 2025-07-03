#[derive(Clone, Debug)]
pub struct AvailableEncodings {
    gzip: bool,
    deflate: bool,
    br: bool,
    zstd: bool,
}

impl AvailableEncodings {
    pub fn from(potential_encodings: &Option<Vec<String>>) -> AvailableEncodings {
        let mut av_enc = AvailableEncodings {
            gzip: false,
            deflate: false,
            br: false,
            zstd: false,
        };

        av_enc.update(potential_encodings);
        av_enc
    }

    pub fn update(&mut self, potential_encodings: &Option<Vec<String>>) {
        if let Some(encodings) = potential_encodings {
            for encoding in encodings {
                match encoding.as_str() {
                    "gzip" => self.gzip = true,
                    "deflate" => self.deflate = true,
                    "br" => self.br = true,
                    "zstd" => self.zstd = true,
                    _ => {}
                }
            }
        }
    }

    pub fn encoding_is_available(&self, encoding: &str) -> bool {
        match encoding {
            "gzip" => self.gzip,
            "deflate" => self.deflate,
            "br" => self.br,
            "zstd" => self.zstd,
            _ => false,
        }
    }
}

pub fn get_encoded_ext(encoding: &str) -> Option<&str> {
    match encoding {
        "gzip" => Some(".gz"),
        "zstd" => Some(".zst"),
        "br" => Some(".br"),
        "deflate" => Some(".zz"),
        _ => None,
    }
}
