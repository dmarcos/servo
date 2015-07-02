/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::AudioParamBinding;
use dom::bindings::codegen::Bindings::AudioParamBinding::AudioParamMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::utils::{Reflector, reflect_dom_object};

use std::cell::RefCell;

#[dom_struct]
pub struct AudioParam {
    reflector_: Reflector,
    value: RefCell<f32>,
}

impl AudioParam {
    fn new_inherited() -> AudioParam {
        AudioParam {
            reflector_: Reflector::new(),
            value: RefCell::new(0f32),
        }
    }

    pub fn new(global: GlobalRef) -> Root<AudioParam> {
        reflect_dom_object(box AudioParam::new_inherited(), global, AudioParamBinding::Wrap)
    }
}

impl<'a> AudioParamMethods for &'a AudioParam {

    fn Value(self) -> Finite<f32> {
        Finite::wrap(*self.value.borrow())
    }

    fn SetValue(self, value: Finite<f32>) -> () {
        *self.value.borrow_mut() = (*value) as f32;
    }

}
