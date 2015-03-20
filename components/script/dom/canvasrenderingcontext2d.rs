/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasWindingRule;
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::codegen::UnionTypes::HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D;
use dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use dom::bindings::error::Error::{IndexSize, TypeError};
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, LayoutJS, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::canvasgradient::{CanvasGradient, CanvasGradientStyle, ToFillOrStrokeStyle};
use dom::htmlcanvaselement::{HTMLCanvasElement, HTMLCanvasElementHelpers};
use dom::imagedata::{ImageData, ImageDataHelpers};

use cssparser::Color as CSSColor;
use cssparser::{Parser, RGBA, ToCss};
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use canvas::canvas_paint_task::{CanvasMsg, CanvasPaintTask, FillOrStrokeStyle};
use canvas::canvas_paint_task::{LinearGradientStyle, RadialGradientStyle};

use std::borrow::ToOwned;
use std::cell::Cell;
use std::num::{Float, ToPrimitive};
use std::sync::mpsc::{channel, Sender};

use collections::string::String;

#[dom_struct]
pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Sender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
    image_smoothing_enabled: Cell<bool>,
    stroke_color: Cell<RGBA>,
    fill_color: Cell<RGBA>,
    transform: Cell<Matrix2D<f32>>,
}

impl CanvasRenderingContext2D {
    fn new_inherited(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
                     -> CanvasRenderingContext2D {
        let black = RGBA {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            renderer: CanvasPaintTask::start(size),
            canvas: JS::from_rooted(canvas),
            image_smoothing_enabled: Cell::new(true),
            stroke_color: Cell::new(black),
            fill_color: Cell::new(black),
            transform: Cell::new(Matrix2D::identity()),
        }
    }

    pub fn new(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
               -> Temporary<CanvasRenderingContext2D> {
        reflect_dom_object(box CanvasRenderingContext2D::new_inherited(global, canvas, size),
                           global, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.renderer.send(CanvasMsg::Recreate(size)).unwrap();
    }

    fn update_transform(&self) {
        self.renderer.send(CanvasMsg::SetTransform(self.transform.get())).unwrap()
    }

    // It is used by DrawImage to calculate the size of the source and destination rectangles based
    // on the drawImage call arguments
    // source rectangle = area of the original image to be copied
    // destination rectangle = area of the destination canvas where the source image is going to be drawn
    #[allow(unused_variables)]
    fn adjust_source_dest_rects(&self,
                  canvas: JSRef<HTMLCanvasElement>,
                  sx: f64, sy: f64, sw: Option<f64>, sh: Option<f64>,
                  dx: Option<f64>, dy: Option<f64>, dw: Option<f64>, dh: Option<f64>) -> (Rect<i32>, Rect<i32>) {
        let context = canvas.get_2d_context().root();
        let renderer = context.r().get_renderer();
        let image_size = canvas.get_size();
        let image_rect = Rect(Point2D(0i32, 0i32), image_size);
        let image_width: f64 = image_size.width.to_f64().unwrap();
        let image_height: f64 = image_size.height.to_f64().unwrap();
        let zero_f64 = 0.0f64;

        // Rules described in the spec:
        // https://html.spec.whatwg.org/multipage/scripting.html#dom-context-2d-drawimage

        // (1) If the sx, sy, sw, and sh arguments are omitted, they must default to 0, 0,
        // the image's intrinsic width in image pixels, and the image's intrinsic height in image pixels, respectively
        // (2) The destination rectangle is the rectangle whose top left corner is (dx, dy)
        // (3) Two - context.drawImage(image, dx, dy) - or
        //     Four arguments provided - context.drawImage(image, dx, dy, dw, dh) -
        // (4) If not specified, the dw and dh arguments must default to the values of sw and sh
        // (5) 8 arguments provided - context.drawImage(image, sx, sy, sw, sh, dx, dy, dw, dh)

        let source_x: f64 = match dx {
            Some(dx) => sx, // (5) drawImage(image, [sx], sy, sw, sh, dx, dy, dw, dh)
            None => zero_f64, // (1) drawImage(image, dx, dy)
        };

        let source_y: f64 = match dy {
            Some(dy) => sy, // (5) drawImage(image, sx, [sy], sw, sh, dx, dy, dw, dh)
            None => zero_f64, // (1) drawImage(image, dx, dy)
        };

        let source_width: f64 = match dw {
            Some(dw) => sw.unwrap(), // (5) drawImage(image, sx, sy, [sw], sh, dx, dy, dw, dh)
            None => image_width, // (1) drawImage(image, dx, dy)
        };

        let source_height: f64 = match dh {
            Some(dh) => sh.unwrap(), // (5) drawImage(image, sx, sy, sw, [sh], dx, dy, dw, dh)
            None => image_height, // (1) drawImage(image, dx, dy)
        };

        // drawImage(image, sx, sy, sw, sh, [dx], dy, dw, dh) or drawImage(image, [sx], sy, sw, sh)
        let dest_x: f64 = dx.unwrap_or(sx); // (2), (3)

        // drawImage(image, sx, sy, sw, sh, dx, [dy], dw, dh) or drawImage(image, sx, [sy], sw, sh)
        let dest_y: f64 = dy.unwrap_or(sy); // (2), (3)

        // drawImage(image, sx, sy, sw, sh, dx, dy, [dw], dh) or
        // drawImage(image, sx, sy, [sw], sh) or
        // drawImage(image, dx, dy) falls back to source_width
        let dest_width: f64 = dw.unwrap_or(sw.unwrap_or(source_width)); // (4), (1)

        // drawImage(image, sx, sy, sw, sh, dx, dy, dw, [dh]) or
        // drawImage(image, sx, sy, sw, [sh]) or
        // drawImage(image, dx, dy) falls back to source_height
        let dest_height: f64 = dh.unwrap_or(sh.unwrap_or(source_height)); // (4), (1)

        // The source rectangle is the rectangle whose corners are the four points (sx, sy),
        // (sx+sw, sy), (sx+sw, sy+sh), (sx, sy+sh).
        let source_rect = Rect(Point2D(source_x.to_i32().unwrap(),
                                       source_y.to_i32().unwrap()),
                               Size2D(source_width.to_i32().unwrap(),
                                      source_height.to_i32().unwrap()));

        // When the source rectangle is outside the source image,
        // the source rectangle must be clipped to the source image
        let source_rect_clipped = source_rect.intersection(&image_rect).unwrap_or(Rect::zero());

        // Width and height ratios between the non clipped and clipped source rectangles
        let width_ratio: f64 = source_rect_clipped.size.width.to_f64().unwrap() / source_rect.size.width.to_f64().unwrap();
        let height_ratio: f64 = source_rect_clipped.size.height.to_f64().unwrap() / source_rect.size.height.to_f64().unwrap();

        // When the source rectangle is outside the source image,
        // the destination rectangle must be clipped in the same proportion.
        let dest_rect_width_scaled: f64 = dest_width * width_ratio;
        let dest_rect_height_scaled: f64 = dest_height * height_ratio;

        // The destination rectangle is the rectangle whose corners are the four points (dx, dy),
        // (dx+dw, dy), (dx+dw, dy+dh), (dx, dy+dh).
        let dest_rect = Rect(Point2D(dest_x.to_i32().unwrap(),
                                     dest_y.to_i32().unwrap()),
                             Size2D(dest_rect_width_scaled.to_i32().unwrap(),
                                    dest_rect_height_scaled.to_i32().unwrap()));

        return (source_rect_clipped, dest_rect)
    }

    //
    // drawImage coordinates explained
    //
    //  Source Image      Destination Canvas
    // +-------------+     +-------------+
    // |             |     |             |
    // |(sx,sy)      |     |(dx,dy)      |
    // |   +----+    |     |   +----+    |
    // |   |    |    |     |   |    |    |
    // |   |    |sh  |---->|   |    |dh  |
    // |   |    |    |     |   |    |    |
    // |   +----+    |     |   +----+    |
    // |     sw      |     |     dw      |
    // |             |     |             |
    // +-------------+     +-------------+
    //
    //
    // The rectangle (sx, sy, sw, sh) from the source image
    // is copied on the rectangle (dx, dy, dh, dw) of the destination canvas
    //
    // https://html.spec.whatwg.org/multipage/scripting.html#dom-context-2d-drawimage
    fn draw_html_canvas_element(&self,
                  canvas: JSRef<HTMLCanvasElement>,
                  sx: f64, sy: f64, sw: Option<f64>, sh: Option<f64>,
                  dx: Option<f64>, dy: Option<f64>, dw: Option<f64>, dh: Option<f64>) -> Fallible<()> {

        // 1. Check the usability of the image argument
        if !canvas.is_valid() {
            return Ok(())
        }

        // 2. Establish the source and destination rectangles
        let (source_rect, dest_rect) = self.adjust_source_dest_rects(canvas, sx, sy, sw, sh, dx, dy, dw, dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Err(IndexSize)
        }

        let smoothing_enabled = self.image_smoothing_enabled.get();
        let canvas_size = canvas.get_size();

        // If the source and target canvas are the same
        let msg = if self.canvas == canvas.unrooted() {
            CanvasMsg::DrawImageSelf(canvas_size, dest_rect, source_rect, smoothing_enabled)
        } else { // Source and target canvases are different
            let context = canvas.get_2d_context().root();
            let renderer = context.r().get_renderer();
            let (sender, receiver) = channel::<Vec<u8>>();
            // Reads pixels from source image
            renderer.send(CanvasMsg::GetImageData(source_rect, canvas_size, sender)).unwrap();
            let imagedata = receiver.recv().unwrap();
            CanvasMsg::DrawImage(imagedata, canvas_size, dest_rect, source_rect, smoothing_enabled)
        };

        self.renderer.send(msg).unwrap();
        Ok(())
    }
}

pub trait CanvasRenderingContext2DHelpers {
    fn get_renderer(&self) -> Sender<CanvasMsg>;
}

impl CanvasRenderingContext2DHelpers for CanvasRenderingContext2D {
    fn get_renderer(&self) -> Sender<CanvasMsg> {
        self.renderer.clone()
    }
}

pub trait LayoutCanvasRenderingContext2DHelpers {
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg>;
}

impl LayoutCanvasRenderingContext2DHelpers for LayoutJS<CanvasRenderingContext2D> {
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg> {
        (*self.unsafe_get()).renderer.clone()
    }
}

impl<'a> CanvasRenderingContext2DMethods for JSRef<'a, CanvasRenderingContext2D> {
    fn Canvas(self) -> Temporary<HTMLCanvasElement> {
        Temporary::new(self.canvas)
    }

    fn Scale(self, x: f64, y: f64) {
        self.transform.set(self.transform.get().scale(x as f32, y as f32));
        self.update_transform()
    }

    fn Translate(self, x: f64, y: f64) {
        self.transform.set(self.transform.get().translate(x as f32, y as f32));
        self.update_transform()
    }

    fn Transform(self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.transform.set(self.transform.get().mul(&Matrix2D::new(a as f32,
                                                                   b as f32,
                                                                   c as f32,
                                                                   d as f32,
                                                                   e as f32,
                                                                   f as f32)));
        self.update_transform()
    }

    fn SetTransform(self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.transform.set(Matrix2D::new(a as f32,
                                         b as f32,
                                         c as f32,
                                         d as f32,
                                         e as f32,
                                         f as f32));
        self.update_transform()
    }

    fn FillRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(CanvasMsg::FillRect(rect)).unwrap();
    }

    fn ClearRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(CanvasMsg::ClearRect(rect)).unwrap();
    }

    fn StrokeRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(CanvasMsg::StrokeRect(rect)).unwrap();
    }

    fn BeginPath(self) {
        self.renderer.send(CanvasMsg::BeginPath).unwrap();
    }

    fn ClosePath(self) {
        self.renderer.send(CanvasMsg::ClosePath).unwrap();
    }

    fn Fill(self, _: CanvasWindingRule) {
        self.renderer.send(CanvasMsg::Fill).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/scripting.html#dom-context-2d-drawimage
    fn DrawImage(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                 dx: f64, dy: f64) -> Fallible<()> {
        match image {
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLCanvasElement(image) => {
                let canvas = image.root();
                return self.draw_html_canvas_element(canvas.r(),
                                                     dx, dy, None, None,
                                                     None, None, None, None)
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eCanvasRenderingContext2D(image) => {
                let image = image.root();
                let context = image.r();
                let canvas = context.Canvas().root();
                return self.draw_html_canvas_element(canvas.r(),
                                                     dx, dy, None, None,
                                                     None, None, None, None)
            }
            _ => {
                Err(TypeError(String::from_str("Unknown type")))
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/scripting.html#dom-context-2d-drawimage
    fn DrawImage_(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                  dx: f64, dy: f64, dw: f64, dh: f64) -> Fallible<()> {
        match image {
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLCanvasElement(image) => {
                let canvas = image.root();
                self.draw_html_canvas_element(canvas.r(),
                                              dx, dy, Some(dw), Some(dh),
                                              None, None, None, None)
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eCanvasRenderingContext2D(image) => {
                let image = image.root();
                let context = image.r();
                let canvas = context.Canvas().root();
                return self.draw_html_canvas_element(canvas.r(),
                                                     dx, dy, None, None,
                                                     None, None, None, None)
            }
            _ => {
                Err(TypeError(String::from_str("Unknown type")))
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/scripting.html#dom-context-2d-drawimage
    fn DrawImage__(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                         sx: f64, sy: f64, sw: f64, sh: f64,
                         dx: f64, dy: f64, dw: f64, dh: f64) -> Fallible<()> {
        match image {
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLCanvasElement(image) => {
                let canvas = image.root();
                self.draw_html_canvas_element(canvas.r(),
                                              sx, sy, Some(sw), Some(sh),
                                              Some(dx), Some(dy), Some(dw), Some(dh))
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eCanvasRenderingContext2D(image) => {
                let image = image.root();
                let context = image.r();
                let canvas = context.Canvas().root();
                self.draw_html_canvas_element(canvas.r(),
                                              sx, sy, Some(sw), Some(sh),
                                              Some(dx), Some(dy), Some(dw), Some(dh))
            }
            _ => {
                Err(TypeError(String::from_str("Unknown type")))
            }
        }
    }

    fn MoveTo(self, x: f64, y: f64) {
        self.renderer.send(CanvasMsg::MoveTo(Point2D(x as f32, y as f32))).unwrap();
    }

    fn LineTo(self, x: f64, y: f64) {
        self.renderer.send(CanvasMsg::LineTo(Point2D(x as f32, y as f32))).unwrap();
    }

    fn QuadraticCurveTo(self, cpx: f64, cpy: f64, x: f64, y: f64) {
        self.renderer.send(CanvasMsg::QuadraticCurveTo(Point2D(cpx as f32, cpy as f32),
                                                       Point2D(x as f32, y as f32))).unwrap();
    }

    fn BezierCurveTo(self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.renderer.send(CanvasMsg::BezierCurveTo(Point2D(cp1x as f32, cp1y as f32),
                                                    Point2D(cp2x as f32, cp2y as f32),
                                                    Point2D(x as f32, y as f32))).unwrap();
    }

    fn Arc(self, x: f64, y: f64, r: f64, start: f64, end: f64, ccw: bool) {
        self.renderer.send(CanvasMsg::Arc(Point2D(x as f32, y as f32), r as f32,
                                          start as f32, end as f32, ccw)).unwrap();
    }

    // https://html.spec.whatwg.org/#dom-context-2d-imagesmoothingenabled
    fn ImageSmoothingEnabled(self) -> bool {
        self.image_smoothing_enabled.get()
    }

    // https://html.spec.whatwg.org/#dom-context-2d-imagesmoothingenabled
    fn SetImageSmoothingEnabled(self, value: bool) -> () {
        self.image_smoothing_enabled.set(value);
    }

    fn StrokeStyle(self) -> StringOrCanvasGradientOrCanvasPattern {
        // FIXME(pcwalton, #4761): This is not spec-compliant. See:
        //
        // https://html.spec.whatwg.org/multipage/scripting.html#serialisation-of-a-colour
        let mut result = String::new();
        self.stroke_color.get().to_css(&mut result).unwrap();
        StringOrCanvasGradientOrCanvasPattern::eString(result)
    }

    fn SetStrokeStyle(self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::eString(string) => {
                match parse_color(string.as_slice()) {
                    Ok(rgba) => {
                        self.stroke_color.set(rgba);
                        self.renderer
                            .send(CanvasMsg::SetStrokeStyle(FillOrStrokeStyle::Color(rgba)))
                            .unwrap();
                    }
                    _ => {}
                }
            }
            _ => {
                // TODO(pcwalton)
            }
        }
    }

    fn FillStyle(self) -> StringOrCanvasGradientOrCanvasPattern {
        // FIXME(pcwalton, #4761): This is not spec-compliant. See:
        //
        // https://html.spec.whatwg.org/multipage/scripting.html#serialisation-of-a-colour
        let mut result = String::new();
        self.stroke_color.get().to_css(&mut result).unwrap();
        StringOrCanvasGradientOrCanvasPattern::eString(result)
    }

    fn SetFillStyle(self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::eString(string) => {
                match parse_color(string.as_slice()) {
                    Ok(rgba) => {
                        self.fill_color.set(rgba);
                        self.renderer
                            .send(CanvasMsg::SetFillStyle(FillOrStrokeStyle::Color(rgba)))
                            .unwrap()
                    }
                    _ => {}
                }
            }
            StringOrCanvasGradientOrCanvasPattern::eCanvasGradient(gradient) => {
                self.renderer.send(CanvasMsg::SetFillStyle(gradient.root().r().to_fill_or_stroke_style())).unwrap();
            }
            _ => {}
        }
    }

    fn CreateImageData(self, sw: f64, sh: f64) -> Fallible<Temporary<ImageData>> {
        if sw == 0.0 || sh == 0.0 {
            return Err(IndexSize)
        }

        Ok(ImageData::new(self.global.root().r(), sw.abs().to_u32().unwrap(), sh.abs().to_u32().unwrap(), None))
    }

    fn CreateImageData_(self, imagedata: JSRef<ImageData>) -> Fallible<Temporary<ImageData>> {
        Ok(ImageData::new(self.global.root().r(), imagedata.Width(), imagedata.Height(), None))
    }

    fn GetImageData(self, sx: f64, sy: f64, sw: f64, sh: f64) -> Fallible<Temporary<ImageData>> {
        if sw == 0.0 || sh == 0.0 {
            return Err(IndexSize)
        }

        let (sender, receiver) = channel::<Vec<u8>>();
        let dest_rect = Rect(Point2D(sx.to_i32().unwrap(), sy.to_i32().unwrap()), Size2D(sw.to_i32().unwrap(), sh.to_i32().unwrap()));
        let canvas_size = self.canvas.root().r().get_size();
        self.renderer.send(CanvasMsg::GetImageData(dest_rect, canvas_size, sender)).unwrap();
        let data = receiver.recv().unwrap();
        Ok(ImageData::new(self.global.root().r(), sw.abs().to_u32().unwrap(), sh.abs().to_u32().unwrap(), Some(data)))
    }

    fn PutImageData(self, imagedata: JSRef<ImageData>, dx: f64, dy: f64) {
        let data = imagedata.get_data_array(&self.global.root().r());
        let image_data_rect = Rect(Point2D(dx.to_i32().unwrap(), dy.to_i32().unwrap()), imagedata.get_size());
        let dirty_rect = None;
        self.renderer.send(CanvasMsg::PutImageData(data, image_data_rect, dirty_rect)).unwrap()
    }

    fn PutImageData_(self, imagedata: JSRef<ImageData>, dx: f64, dy: f64,
                     dirtyX: f64, dirtyY: f64, dirtyWidth: f64, dirtyHeight: f64) {
        let data = imagedata.get_data_array(&self.global.root().r());
        let image_data_rect = Rect(Point2D(dx.to_i32().unwrap(), dy.to_i32().unwrap()),
                                   Size2D(imagedata.Width().to_i32().unwrap(),
                                          imagedata.Height().to_i32().unwrap()));
        let dirty_rect = Some(Rect(Point2D(dirtyX.to_i32().unwrap(), dirtyY.to_i32().unwrap()),
                                   Size2D(dirtyWidth.to_i32().unwrap(),
                                          dirtyHeight.to_i32().unwrap())));
        self.renderer.send(CanvasMsg::PutImageData(data, image_data_rect, dirty_rect)).unwrap()
    }

    fn CreateLinearGradient(self, x0: f64, y0: f64, x1: f64, y1: f64) -> Fallible<Temporary<CanvasGradient>> {
        if [x0, y0, x1, y1].iter().any(|x| x.is_nan() || x.is_infinite()) {
            return Err(TypeError("One of the arguments of createLinearGradient() is not a finite floating-point value.".to_owned()));
        }
        Ok(CanvasGradient::new(self.global.root().r(),
                               CanvasGradientStyle::Linear(LinearGradientStyle::new(x0, y0, x1, y1, Vec::new()))))
    }

    fn CreateRadialGradient(self, x0: f64, y0: f64, r0: f64, x1: f64, y1: f64, r1: f64) -> Fallible<Temporary<CanvasGradient>> {
        if [x0, y0, r0, x1, y1, r1].iter().any(|x| x.is_nan() || x.is_infinite()) {
            return Err(TypeError("One of the arguments of createRadialGradient() is not a finite floating-point value.".to_owned()));
        }
        Ok(CanvasGradient::new(self.global.root().r(),
                               CanvasGradientStyle::Radial(RadialGradientStyle::new(x0, y0, r0, x1, y1, r1, Vec::new()))))
    }
}

#[unsafe_destructor]
impl Drop for CanvasRenderingContext2D {
    fn drop(&mut self) {
        self.renderer.send(CanvasMsg::Close).unwrap();
    }
}

pub fn parse_color(string: &str) -> Result<RGBA,()> {
    match CSSColor::parse(&mut Parser::new(string.as_slice())) {
        Ok(CSSColor::RGBA(rgba)) => Ok(rgba),
        _ => Err(()),
    }
}

// Used by drawImage to determine if a source or destination rectangle is valid
// Origin coordinates and size cannot be negative. Size has to be greater than zero
fn is_rect_valid(rect: Rect<i32>) -> bool {
    rect.origin.x >= 0 && rect.origin.y >= 0 && rect.size.width > 0 && rect.size.height > 0
}
