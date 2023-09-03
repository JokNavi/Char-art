use std::collections::HashMap;

use image::{GrayImage, Luma};
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};

const DEFAULT_PRINTABLE_CHARACTERS: &str = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ ";
const DEFAULT_BRIGHTNESSES: [u8; 95] = [
    22, 25, 59, 55, 48, 65, 13, 27, 28, 41, 34, 8, 15, 5, 23, 64, 33, 50, 50, 53, 53, 53, 38, 61,
    55, 11, 13, 29, 33, 29, 34, 61, 53, 71, 46, 62, 56, 46, 57, 58, 48, 36, 60, 36, 75, 73, 56, 53,
    57, 64, 55, 38, 52, 48, 79, 54, 41, 51, 35, 23, 35, 32, 12, 10, 49, 56, 37, 56, 47, 43, 54, 52,
    39, 34, 53, 44, 61, 44, 43, 47, 48, 28, 42, 37, 42, 34, 52, 40, 38, 43, 32, 23, 32, 24, 0,
];

#[derive(Debug, PartialEq, Clone)]
pub struct KeyBrightnesses {
    keys: String,
    brightnesses: Vec<u8>,
}

impl KeyBrightnesses {
    pub const KEY_REPETITION: u8 = 3;
    pub const KEY_WIDTH_MULTIPLIER: u8 = 2;
    pub const CHUNK_WIDTH_KEY_AMOUNT: usize =
        (Self::KEY_REPETITION as u16 * Self::KEY_WIDTH_MULTIPLIER as u16) as usize;
    const KEY_COLOR: Luma<u8> = Luma([255]);

    pub fn new(keys: &str, font: Font, scale: Scale) -> Self {
        if keys.contains(' ') {
            panic!("Keys cannot contain spaces.");
        }
        Self {
            keys: keys.to_string(),
            brightnesses: Self::keys_average_brightnesses(keys, font, scale),
        }
    }

    fn keys_average_brightnesses(keys: &str, font: Font, scale: Scale) -> Vec<u8> {
        let key_chunk_rows: Vec<String> = keys
            .chars()
            .map(|key| key.to_string().repeat(Self::CHUNK_WIDTH_KEY_AMOUNT))
            .collect();
        let mut key_brightnesess: Vec<u8> = Vec::with_capacity(keys.len());
        for key_chunk_row in key_chunk_rows.iter() {
            let (chunk_row_width, chunk_row_height) = text_size(scale, &font, key_chunk_row);
            let mut image = GrayImage::new(
                chunk_row_width as u32,
                chunk_row_height as u32 * Self::KEY_REPETITION as u32,
            );
            for y in 0..Self::KEY_REPETITION {
                draw_text_mut(
                    &mut image,
                    Self::KEY_COLOR,
                    0,
                    y as i32 * chunk_row_height,
                    scale,
                    &font,
                    key_chunk_row,
                );
            }
            key_brightnesess.push(Self::average_brightness(&image));
        }
        key_brightnesess
    }

    fn average_brightness(image: &GrayImage) -> u8 {
        (image
            .pixels()
            .map(|pixel| pixel.0[0] as usize)
            .sum::<usize>()
            / image.len())
        .try_into()
        .unwrap()
    }

    pub fn brightnesses(&self) -> &[u8] {
        &self.brightnesses
    }

    pub fn keys(&self) -> &str {
        &self.keys
    }

    pub fn as_tuple(&self) -> Vec<(u8, char)> {
        <Vec<(u8, char)>>::from(self)
    }

    pub fn as_hash_map(&self) -> HashMap<u8, char> {
        HashMap::<u8, char>::from(self)
    }
}

impl Default for KeyBrightnesses {
    fn default() -> Self {
        Self {
            keys: DEFAULT_PRINTABLE_CHARACTERS.to_owned(),
            brightnesses: DEFAULT_BRIGHTNESSES.to_vec(),
        }
    }
}

impl From<&KeyBrightnesses> for Vec<(u8, char)> {
    fn from(average_key_brightnesses: &KeyBrightnesses) -> Self {
        average_key_brightnesses
            .brightnesses
            .iter()
            .copied()
            .zip(average_key_brightnesses.keys.chars())
            .map(|(brigthness, key)| (brigthness, key))
            .collect()
    }
}

impl From<&KeyBrightnesses> for HashMap<u8, char> {
    fn from(average_key_brightnesses: &KeyBrightnesses) -> Self {
        HashMap::from_iter(<Vec<(u8, char)>>::from(average_key_brightnesses).into_iter())
    }
}

#[cfg(test)]
mod key_brightnesses_tests {
    const PRINTABLE_CHARACTERS: &str = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    use super::KeyBrightnesses;
    use image::{GrayImage, Luma};
    use imageproc::{drawing::draw_filled_rect, rect::Rect};
    use rusttype::{Font, Scale};
    use std::collections::HashMap;

    fn get_font() -> Font<'static> {
        let font_bytes = include_bytes!("/home/joknavi/.local/share/fonts/RobotoMono-Regular.ttf");
        Font::try_from_bytes(font_bytes).unwrap()
    }

    #[test]
    fn average_brightness() {
        let bright_image = draw_filled_rect(
            &GrayImage::new(100, 100),
            Rect::at(0, 0).of_size(100, 100),
            KeyBrightnesses::KEY_COLOR,
        );
        let dark_image = draw_filled_rect(
            &GrayImage::new(100, 100),
            Rect::at(0, 0).of_size(100, 100),
            Luma([0]),
        );
        assert_eq!(KeyBrightnesses::average_brightness(&dark_image), 0);
        assert_eq!(KeyBrightnesses::average_brightness(&bright_image), 255);
    }

    #[test]
    fn new() {
        let scale = Scale::uniform(12.0);
        let font = get_font();
        let average_key_brightnesess =
            KeyBrightnesses::new(PRINTABLE_CHARACTERS, font.clone(), scale);
        let key_brightnesess =
            KeyBrightnesses::keys_average_brightnesses(PRINTABLE_CHARACTERS, font, scale);
        assert_eq!(average_key_brightnesess.brightnesses, key_brightnesess);
    }

    #[test]
    fn as_tuple() {
        let average_key_brightnesess = KeyBrightnesses::default();
        assert_eq!(
            <Vec<(u8, char)>>::from(&average_key_brightnesess)[0],
            (
                average_key_brightnesess.brightnesses[0],
                average_key_brightnesess.keys.chars().next().unwrap()
            )
        )
    }

    #[test]
    fn as_hash_map() {
        let average_key_brightnesess = KeyBrightnesses::default();
        let first_tuple = average_key_brightnesess.as_tuple()[0];
        let binding = <HashMap<u8, char>>::from(&average_key_brightnesess);
        let hash_map_tuple = binding.get_key_value(&first_tuple.0).unwrap();
        assert_eq!(hash_map_tuple, (&first_tuple.0, &first_tuple.1))
    }
}
