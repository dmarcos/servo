/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::PeriodicWaveBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};

#[dom_struct]
pub struct PeriodicWave {
    reflector_: Reflector,
    id: u32
}

impl PeriodicWave {
    fn new_inherited(id: u32) -> PeriodicWave {
        PeriodicWave {
            reflector_: Reflector::new(),
            id: id,
        }
    }

    pub fn new(global: GlobalRef, id: u32) -> Root<PeriodicWave> {
        reflect_dom_object(box PeriodicWave::new_inherited(id), global, PeriodicWaveBinding::Wrap)
    }
}
