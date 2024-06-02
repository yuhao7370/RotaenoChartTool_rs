// TextureManager.rs
use macroquad::{prelude::*};
use image::{DynamicImage, ImageBuffer, Rgba};
use macroquad::texture::Texture2D;

pub struct TextureManager {
    texture: Texture2D,
    width: f32,
    height: f32,
}

impl TextureManager {
    pub async fn new(path: &str) -> Result<Self, String> {
        let texture: Texture2D = load_texture(path).await.map_err(|_| format!("Failed to load texture from {}", path))?;
        let width: f32 = texture.width();
        let height: f32 = texture.height();
        Ok(Self { texture, width, height })
    }

    pub async fn new_blurred(path: &str, blur_radius: f32) -> Result<Self, String> {
        let image: DynamicImage = image::open(path).map_err(|_| format!("Failed to load image from {}", path))?;
    
        let width: u32 = image.width();
        let height: u32 = image.height();
        let size = (width * height) as usize;

        let mut blurred_rgb = image.to_rgb8();
        let mut vec = unsafe { Vec::from_raw_parts(std::mem::transmute(blurred_rgb.as_mut_ptr()), size, size) };
        fastblur::gaussian_blur(&mut vec, width as _, height as _, blur_radius);
        std::mem::forget(vec);
        let mut blurred = Vec::with_capacity(size * 4);
        for input in blurred_rgb.chunks_exact(3) {
            blurred.extend_from_slice(input);
            blurred.push(255);
        }
    
        let blurred_texture: Texture2D = Texture2D::from_image(&Image {
            width: width as _,
            height: height as _,
            bytes: blurred,
        });

        Ok(Self { texture: blurred_texture, width: width as f32, height: height as f32 })
    }


    pub fn texture(&self) -> &Texture2D {
        &self.texture
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn crop(&self, target_width: f32, target_height: f32) -> DrawTextureParams {
        let target_aspect_ratio = target_width / target_height;
        let (crop_width, crop_height) = if self.width / self.height > target_aspect_ratio {
            // If the image is wider than the target aspect ratio, constrain by height
            (self.height * target_aspect_ratio, self.height)
        } else {
            // If the image is taller than the target aspect ratio, constrain by width
            (self.width, self.width / target_aspect_ratio)
        };
    
        let start_x = (self.width - crop_width) / 2.0;
        let start_y = (self.height - crop_height) / 2.0;
    
        DrawTextureParams {
            source: Some(Rect::new(start_x, start_y, crop_width, crop_height)),
            dest_size: Some(Vec2::new(target_width, target_height)),
            ..Default::default()
        }
    }

    pub fn draw_cropped(&self, x: f32, y: f32, target_width: f32, target_height: f32, rotation: f32) {
        let mut params = self.crop(target_width, target_height);
        params.rotation = rotation *std::f32::consts::PI / 180.0;
        draw_texture_ex(self.texture(),x,y,WHITE,params,);
    }
}