use js_sys::{Array, Uint8Array};
use palette::{Pixel, Srgba};
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, Clamped, JsValue};
use web_sys::{Blob, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, Url};
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::{html, ChangeData, Component, ComponentLink, Html, NodeRef, ShouldRender};

pub struct App {
    link: ComponentLink<Self>,
    image_loaded_closure: Closure<dyn FnMut(JsValue)>,
    image_error_closure: Closure<dyn FnMut(JsValue)>,
    tasks: Vec<ReaderTask>,
    file_info: Option<FileInfo>,
    c1_node_ref: NodeRef,
    c1_context_2d: Option<CanvasRenderingContext2d>,
    c1_canvas: Option<HtmlCanvasElement>,
    c2_node_ref: NodeRef,
    c2_context_2d: Option<CanvasRenderingContext2d>,
    c2_canvas: Option<HtmlCanvasElement>,
    state: AppState,
    error_log: Vec<String>,
    position_info: PositionInfo,
}

pub struct PositionInfo {
    canv_height: usize,
    image_canv_width: usize,
}

impl PositionInfo {
    fn new() -> Self {
        Self {
            image_canv_width: 300,
            canv_height: 200,
        }
    }

    fn update_window_size(&mut self, w: i32, h: i32) {
        log::info!("window size: {}x{}", w, h);
    }

    /// An image has been loaded, recalculate various sizing info.
    fn update_for_image(&mut self, img: &HtmlImageElement) {
        log::info!("new image size");
    }

    /// The width of the images in the canvas.
    fn image_canv_width(&self) -> usize {
        self.image_canv_width
    }
    /// The height of the images in the canvas.
    fn image_canv_height(&self) -> usize {
        self.canv_height * 2
    }
    /// The width of the canvas.
    fn canv_width(&self) -> usize {
        self.image_canv_width
    }
    /// The height of the canvas.
    fn canv_height(&self) -> usize {
        self.canv_height
    }
}

pub enum AppState {
    Ready,
    ReadingFile,
    DecodingImage(FileInfo),
}

pub struct FileInfo {
    file_data: FileData,
    img: HtmlImageElement,
}

pub enum Msg {
    FileLoaded(FileData),
    Files(Vec<File>),
    ImageLoaded,
    ImageErrored(String),
    // Resize(WindowDimensions),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link2 = link.clone();
        let image_loaded_closure = Closure::wrap(Box::new(move |_| {
            link2.send_message(Msg::ImageLoaded);
        }) as Box<dyn FnMut(JsValue)>);

        let link2 = link.clone();
        let image_error_closure = Closure::wrap(Box::new(move |arg| {
            // let err_str = format!("Failed to load image.{:?}", arg);
            let err_str = "Failed to load image.".into();
            log::error!("{:?}", arg);
            link2.send_message(Msg::ImageErrored(err_str));
        }) as Box<dyn FnMut(_)>);

        App {
            link,
            image_loaded_closure,
            image_error_closure,
            tasks: vec![],
            // _resize_task: resize_task,
            c1_node_ref: NodeRef::default(),
            c1_context_2d: None,
            c1_canvas: None,
            c2_node_ref: NodeRef::default(),
            c2_context_2d: None,
            c2_canvas: None,
            file_info: None,
            state: AppState::Ready,
            error_log: vec![],
            position_info: PositionInfo::new(),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        // Once rendered, store references for the canvas and 2D context. These can be used for
        // resizing the rendering area when the window or canvas element are resized.

        let document = yew::utils::document();
        let div_wrapper: web_sys::Element = document
            .query_selector("#colorswitch-canvas-div")
            .unwrap()
            .unwrap();

        // Get inner dimensions of div containing canvases.
        let div_w = div_wrapper.client_width();
        let div_h = div_wrapper.client_height();

        self.position_info.update_window_size(div_w, div_h);

        let canvas = self.c1_node_ref.cast::<HtmlCanvasElement>().unwrap();

        let context_2d = CanvasRenderingContext2d::from(JsValue::from(
            canvas.get_context("2d").unwrap().unwrap(),
        ));

        self.c1_canvas = Some(canvas);
        self.c1_context_2d = Some(context_2d);

        let canvas = self.c2_node_ref.cast::<HtmlCanvasElement>().unwrap();

        let context_2d = CanvasRenderingContext2d::from(JsValue::from(
            canvas.get_context("2d").unwrap().unwrap(),
        ));

        self.c2_canvas = Some(canvas);
        self.c2_context_2d = Some(context_2d);
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ImageLoaded => {
                // The image has finished decoding and we can display it now.
                let old_state = std::mem::replace(&mut self.state, AppState::Ready);

                if let AppState::DecodingImage(file_info) = old_state {
                    self.position_info.update_for_image(&file_info.img);

                    // The image has finished loading (decoding).
                    if let (Some(ctx1), Some(ctx2)) =
                        (self.c1_context_2d.as_ref(), self.c2_context_2d.as_ref())
                    {
                        // Draw the original image on the canvas.
                        ctx1.draw_image_with_html_image_element_and_dw_and_dh(
                            &file_info.img,
                            0.0,
                            0.0,
                            self.position_info.image_canv_width() as f64,
                            self.position_info.image_canv_height() as f64,
                        )
                        .unwrap();

                        // Read the original image data from the canvas.
                        let image_data: web_sys::ImageData = ctx1
                            .get_image_data(
                                0.0,
                                0.0,
                                self.position_info.image_canv_width() as f64,
                                self.position_info.image_canv_height() as f64,
                            )
                            .unwrap();

                        let w = image_data.width();
                        let h = image_data.height();
                        debug_assert!(w as usize == self.position_info.image_canv_width());
                        debug_assert!(h as usize == self.position_info.image_canv_height());

                        let new_data = {
                            let mut data = image_data.data();

                            let color_buffer: &mut [Srgba<u8>] =
                                Pixel::from_raw_slice_mut(data.as_mut_slice());
                            for pix in color_buffer.iter_mut() {
                                let rgb: palette::rgb::Rgb<palette::encoding::Srgb, u8> = pix.color;
                                let rgb_f32: palette::rgb::Rgb<palette::encoding::Srgb, f32> =
                                    rgb.into_format();

                                use palette::ConvertInto;
                                let mut hsv_f32: palette::Hsv<palette::encoding::Srgb, f32> =
                                    rgb_f32.convert_into();
                                hsv_f32.saturation *= 4.0;
                                hsv_f32.hue =
                                    palette::RgbHue::from_degrees(hsv_f32.hue.to_degrees() + 180.0);

                                let rgb_f32: palette::rgb::Rgb<palette::encoding::Srgb, f32> =
                                    hsv_f32.convert_into();
                                let rgb_u8: palette::rgb::Rgb<palette::encoding::Srgb, u8> =
                                    rgb_f32.into_format();
                                pix.color = rgb_u8;
                            }

                            let new_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                                Clamped(data.as_mut_slice()),
                                w,
                                h,
                            )
                            .unwrap();

                            new_data
                        };
                        ctx2.put_image_data(&new_data, 0.0, 0.0).unwrap();
                    }
                    self.file_info = Some(file_info);
                }
            }
            Msg::ImageErrored(err_str) => {
                // The image was not decoded due to an error.
                self.error_log.push(err_str);
                self.state = AppState::Ready;
            }
            Msg::FileLoaded(file_data) => {
                // The bytes of the file have been read.

                // Convert to a Uint8Array and initiate the image decoding.
                let buffer = Uint8Array::from(file_data.content.as_slice());
                let buffer_val: &JsValue = buffer.as_ref();
                let parts = Array::new_with_length(1);
                parts.set(0, buffer_val.clone());
                let blob = Blob::new_with_u8_array_sequence(parts.as_ref()).unwrap();
                let img = HtmlImageElement::new().unwrap();

                img.set_onload(Some(self.image_loaded_closure.as_ref().unchecked_ref()));

                img.set_onerror(Some(self.image_error_closure.as_ref().unchecked_ref()));

                img.set_src(&Url::create_object_url_with_blob(&blob).unwrap());

                self.state = AppState::DecodingImage(FileInfo { file_data, img });
            }
            Msg::Files(files) => {
                // The user has selected file(s).
                self.error_log.clear();

                self.state = AppState::ReadingFile;

                for file in files.into_iter() {
                    let task = {
                        let callback = self.link.callback(Msg::FileLoaded);
                        ReaderService::read_file(file, callback).unwrap()
                    };
                    self.tasks.push(task);
                }
            }
        }
        true
    }

    fn view(&self) -> Html {
        let state = match self.state {
            AppState::Ready => "Ready",
            AppState::ReadingFile => "Reading file",
            AppState::DecodingImage(_) => "Decoding image",
        };
        html! {
            <div class="container">

                <div>
                    <p>{ state }</p>
                    <p>{"Choose an image file to colorswitch."}</p>
                    <input type="file" onchange=self.link.callback(move |value| {
                            let mut result = Vec::new();
                            if let ChangeData::Files(files) = value {
                                let files = js_sys::try_iter(&files)
                                    .unwrap()
                                    .unwrap()
                                    .into_iter()
                                    .map(|v| File::from(v.unwrap()));
                                result.extend(files);
                            }
                            Msg::Files(result)
                        })/>
                </div>

                { self.view_file_info() }
                <div id="colorswitch-canvas-div">
                    <div id="colorswitch-canvas-container">
                        <canvas class="im-canv" ref={self.c1_node_ref.clone()}, width={self.position_info.canv_width()}, height={self.position_info.canv_height()} />
                        <canvas class="im-canv" ref={self.c2_node_ref.clone()}, width={self.position_info.canv_width()}, height={self.position_info.canv_height()} />
                    </div>
                </div>
                { self.view_errors() }

                <div class="info">
                    <p>{ "Source code " }<a href="https://github.com/strawlab/colorswitch">{ "github.com/strawlab/colorswitch" }</a></p>
                </div>
            </div>
        }
    }
}

fn render_error(err_str: &String) -> Html {
    html! {
        <p>{format!("ERROR: {}",err_str)}</p>
    }
}

impl App {
    fn view_file_info(&self) -> Html {
        if let Some(file_info) = &self.file_info {
            html! {
                <p>{file_info.file_data.name.as_str()}</p>
            }
        } else {
            html! {}
        }
    }

    fn view_errors(&self) -> Html {
        if self.error_log.len() > 0 {
            html! {
                <div>
                    { for self.error_log.iter().map(render_error)}
                </div>
            }
        } else {
            html! {}
        }
    }
}
