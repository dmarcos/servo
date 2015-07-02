/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::audioparam::AudioParam;

use dom::bindings::codegen::Bindings::OscillatorNodeBinding;
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorType;
use dom::bindings::codegen::Bindings::OscillatorNodeBinding::OscillatorNodeMethods;
use dom::bindings::codegen::InheritTypes::OscillatorNodeDerived;

use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::num::Finite;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::eventtarget::{EventTarget};

use cult::{AudioStream, CubebContext, CUBEB_SAMPLE_FLOAT32NE, DataCallback};

use js::jsapi::JSTracer;

use std::cell::RefCell;
use std::rc::Rc;
use std::f32;
use std::thread;


impl JSTraceable for AudioStream {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

#[dom_struct]
pub struct OscillatorNode {
    reflector_: Reflector,
    t: RefCell<OscillatorType>,
    audio_param: JS<AudioParam>,
    audio_stream: RefCell<AudioStream>,
}

impl OscillatorNodeDerived for EventTarget {
    fn is_oscillatornode(&self) -> bool {
        true
    }
}

impl OscillatorNode {
    fn new_inherited(global: GlobalRef) -> OscillatorNode {
        let ctx: Rc<CubebContext> = Rc::new(CubebContext::new("rust-cubeb"));
        OscillatorNode {
          reflector_: Reflector::new(),
          t: RefCell::new(OscillatorType::Sine),
          audio_param: JS::from_ref(AudioParam::new(global).r()),
          audio_stream: RefCell::new(AudioStream::new(ctx.clone())),
        }
    }

    pub fn new(global: GlobalRef) -> Root<OscillatorNode> {
        reflect_dom_object(box OscillatorNode::new_inherited(global), global, OscillatorNodeBinding::Wrap)
    }

    pub fn sine(&self) {
      let mut phase: Box<f32> = Box::new(0.0);

      let cb: DataCallback = Box::new(move |buffer: &mut [f32]| {
        let w = f32::consts::PI * 2.0 * 440. / (44100 as f32);
        for i in 0 .. buffer.len() {
          for j in (0..1) {
            buffer[i + j] = (*phase).sin();
          }
          (*phase) += w;
        }
        assert!(buffer.len() != 0);
        buffer.len() as i32
      });

      self.audio_stream.borrow_mut().init(44100, 1, CUBEB_SAMPLE_FLOAT32NE, cb, "rust-cubeb-stream0");
    }
}

impl<'a> OscillatorNodeMethods for &'a OscillatorNode {

    fn Type(self) -> OscillatorType {
        *self.t.borrow()
    }

    fn SetType(self, value: OscillatorType) -> ErrorResult {
        *self.t.borrow_mut() = value;
        Ok(())
    }

    fn Frequency(self) -> Root<AudioParam> {
        self.audio_param.root()
    }

    fn Start(self, when: Option<Finite<f64>>) -> Fallible<()> {
        self.sine();
        self.audio_stream.borrow().start();
        Ok(())
    }

    fn Stop(self, when: Option<Finite<f64>>) -> Fallible<()> {
        self.audio_stream.borrow().stop();
        Ok(())
    }

}

