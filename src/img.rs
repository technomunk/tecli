//! Image manipulation facilitated by the CLI.

use clap::{Parser, Subcommand};

pub use color::Color;

/// Manipulate background images.
#[derive(Debug, Subcommand)]
pub enum Command {
    Seed(SeedArgs),
    Update(UpdateArgs),
}

mod color {
    use image::Rgb;

    /// A color that can be converted to and from a string representation.
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Color(Rgb<u8>);

    impl Into<Rgb<u8>> for Color {
        fn into(self) -> Rgb<u8> {
            self.0
        }
    }

    impl From<Rgb<u8>> for Color {
        fn from(pixel: Rgb<u8>) -> Self {
            Self(pixel)
        }
    }

    impl std::fmt::Display for Color {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "#{:2x}{:2x}{:2x}", self.0[0], self.0[1], self.0[2])
        }
    }

    impl std::str::FromStr for Color {
        type Err = std::num::ParseIntError;

        fn from_str(mut s: &str) -> Result<Self, Self::Err> {
            s = s.trim_matches('#');
            let r = u8::from_str_radix(&s[0..2], 16)?;
            let g = u8::from_str_radix(&s[2..4], 16)?;
            let b = u8::from_str_radix(&s[4..6], 16)?;
            Ok(Color(Rgb([r, g, b])))
        }
    }

    impl Color {
        /// Find the inverse color of self.
        pub fn inverse(self) -> Color {
            let [r, g, b] = self.0 .0;
            Color(Rgb([255 - r, 255 - g, 255 - b]))
        }
    }
}

mod text {
    //! Producing an image of a text.
    use std::error::Error;

    use font_kit::canvas::{Canvas, Format};
    use font_kit::font::Font;
    use pathfinder_geometry::rect::RectI;
    use pathfinder_geometry::vector::vec2i;


    /// Draw provided string of text with a random font and a random size.
    pub fn draw(s: &str, min_point: u32, max_size: (u32, u32)) -> Result<(), TextError> {
        let font = pick_random_font()?;
        font.load_font_table(table_tag)
        Ok(())
    }

    pub fn draw_with_font(s: &str, font: &Font) -> Result<(), TextError> {
        let (w, h) = text_dimensions(s, font)?;
        let mut canvas = Canvas::new(vec2i(w, h), Format::Rgba32);
        for ch in s.chars() {
            let glyph = font.glyph_for_char(ch).ok_or(TextError::MissingGlyphError(ch))?;
            font.rasterize_glyph(
                &mut canvas,
                glyph,
                point_size,
                transform,
                hinting_options,
                rasterization_options,
            )?;
        }
        Ok(())
    }

    pub fn pick_random_font() -> Result<Font, TextError> {
        let fonts = font_kit::source::SystemSource::new().all_fonts()?;
        let index: usize = rand::random();
        let font = Font::from_handle(&fonts[index % fonts.len()])?;
        Ok(font)
    }

    #[derive(Debug)]
    pub enum TextError {
        FontSelectionError(font_kit::error::SelectionError),
        FontLoadingError(font_kit::error::FontLoadingError),
        GlyphLoadingError(font_kit::error::GlyphLoadingError),
        MissingGlyphError(char),
    }

    fn text_dimensions(s: &str, font: &Font) -> Result<(i32, i32), TextError> {
        let mut total_bounds = RectI::new(vec2i(0, 0), vec2i(0, 0));
        let mut cursor = 0;
        for ch in s.chars() {
            let glyph = font.glyph_for_char(ch).ok_or(TextError::MissingGlyphError(ch))?;
            let bounds = font.typographic_bounds(glyph)?.to_i32();

            total_bounds.0[0] = total_bounds.0[0].min(cursor + bounds.min_x());
            total_bounds.0[1] = total_bounds.0[1].min(bounds.min_y());
            total_bounds.0[2] = total_bounds.0[2].max(cursor + bounds.max_x());
            total_bounds.0[3] = total_bounds.0[3].max(bounds.max_y());

            cursor += font.advance(glyph)?.0[0] as i32;
        }
        Ok((total_bounds.width(), total_bounds.height()))
    }

    impl Error for TextError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::FontSelectionError(err) => Some(err),
                Self::FontLoadingError(err) => Some(err),
                Self::GlyphLoadingError(err) => Some(err),
                Self::MissingGlyphError(_) => None,
            }
        }
    }

    impl std::fmt::Display for TextError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::FontSelectionError(err) => err.fmt(f),
                Self::FontLoadingError(err) => err.fmt(f),
                Self::GlyphLoadingError(err) => err.fmt(f),
                Self::MissingGlyphError(ch) => write!(f, "Did not find glyph for '{}'", ch),
            }
        }
    }

    impl From<font_kit::error::SelectionError> for TextError {
        fn from(err: font_kit::error::SelectionError) -> Self {
            Self::FontSelectionError(err)
        }
    }

    impl From<font_kit::error::FontLoadingError> for TextError {
        fn from(err: font_kit::error::FontLoadingError) -> Self {
            Self::FontLoadingError(err)
        }
    }

    impl From<font_kit::error::GlyphLoadingError> for TextError {
        fn from(err: font_kit::error::GlyphLoadingError) -> Self {
            Self::GlyphLoadingError(err)
        }
    }
}

/// Seed a new background image that can be updated afterwards.
#[derive(Debug, Parser)]
pub struct SeedArgs {
    #[clap(short = 'W', long, default_value_t = 1920)]
    width: u16,
    #[clap(short = 'H', long, default_value_t = 1080)]
    height: u16,
    #[clap(short, long, default_value = "#FFFFFF")]
    background: Color,
}

/// Update an existing background image.
#[derive(Debug, Parser)]
pub struct UpdateArgs {}

/// Process the img subcommand.
pub fn command(arg: Command) {
    match arg {
        Command::Seed(args) => seed(args),
        Command::Update(args) => update(args),
    }
}

fn seed(args: SeedArgs) {
    let mut generated_image = image::RgbImage::from_pixel(
        args.width as u32,
        args.height as u32,
        args.background.into(),
    );

    imageproc::drawing::draw_text_mut(
        &mut generated_image,
        args.background.inverse().into(),
        (args.width / 2) as i32,
        (args.height / 2) as i32,
        Scale{1.0, 1.0},
        font,
        text,
    );

    generated_image
        .save("test.png")
        .expect("Failed to save seeded image");
}

fn update(_: UpdateArgs) {
    println!("updating image")
}
