use std::io::{Cursor, Read};

use zaplib::*;

use image::{codecs::jpeg::JpegDecoder, ImageDecoder};

static DUMMY_SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            fn pixel() -> vec4 {
                return vec4(0., 0., 0., 0.);
            }
            "#
        ),
    ],
    ..Shader::DEFAULT
};

static IMAGE_SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            texture texture: texture2D;

            fn pixel() -> vec4 {
                // `1.0-pos.y` to turn it the right way up in Wasm.
                // TODO(JP): Fix https://github.com/Zaplib/zaplib/issues/160
                let color = sample2d(texture, vec2(pos.x, 1.0-pos.y)).rgba;
                return vec4(color);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

struct JpegImage {
    texture: Texture,
}

impl JpegImage {
    fn new(bytes: Vec<u8>, cx: &mut Cx) -> JpegImage {
        // TODO(JP): replace with Zaplib API; see https://github.com/Zaplib/zaplib/issues/161
        let decoder = JpegDecoder::new(Cursor::new(bytes)).expect("Could not decode image");
        let mut img_bytes = vec![0; decoder.total_bytes() as usize];

        let (width, height) = decoder.dimensions();
        decoder.read_image(&mut img_bytes).expect("Could not decode image");

        let mut texture = Texture::default();
        let texture_handle = texture.get_with_dimensions(cx, width as usize, height as usize);
        let texture_img = texture_handle.get_image_mut(cx);

        let img_buffer = image::RgbImage::from_raw(width, height, img_bytes).expect("Could not create image");

        let mut index = 0;
        for h in 0..height {
            for w in 0..width {
                let pixel = img_buffer.get_pixel(w, h);
                let image::Rgb(data) = *pixel;
                *(texture_img.get_mut(index)).unwrap() = u32::from_le_bytes([data[0], data[1], data[2], 0xff]);
                index += 1;
            }
        }

        JpegImage { texture }
    }

    fn draw(&mut self, cx: &mut Cx) {
        let texture_handle = self.texture.unwrap_texture_handle();
        let area = cx.add_instances(&IMAGE_SHADER, &[QuadIns::from_rect(cx.get_box_rect())]);
        area.write_texture_2d(cx, "texture", texture_handle);

        // Dummy shader call to prevent texture batching
        // TODO(JP): Fix https://github.com/Zaplib/zaplib/issues/156
        cx.add_instances(&DUMMY_SHADER, &[QuadIns::default()]);
    }
}

#[derive(Default)]
struct ImageExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,

    images: Vec<JpegImage>,
}

impl ImageExampleApp {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn read_image(&mut self, cx: &mut Cx, path: &str) {
        match UniversalFile::open(path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).expect("Could not read file");
                self.images.push(JpegImage::new(buf, cx));
            }
            Err(msg) => {
                log!("Error: {:?}", msg);
            }
        }
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        match event {
            Event::Construct => {
                self.read_image(cx, "zaplib/examples/example_image/data/img1.jpg");
                self.read_image(cx, "zaplib/examples/example_image/data/img2.jpg");
            }
            _ => {}
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));
        self.main_view.begin_view(cx, LayoutSize::FILL);
        cx.begin_padding_box(Padding::top(30.));

        cx.begin_column(Width::Fill, Height::Fill);

        let total_height = cx.get_height_left();
        for image in self.images.iter_mut() {
            cx.begin_column(Width::Fill, Height::Fix(0.5 * total_height));
            image.draw(cx);
            cx.end_column();
        }
        cx.end_column();

        cx.end_padding_box();
        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(ImageExampleApp);
