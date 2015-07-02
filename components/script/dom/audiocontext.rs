/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::AudioContextBinding;
use dom::bindings::codegen::Bindings::AudioContextBinding::AudioContextMethods;

use dom::audiodestinationnode::AudioDestinationNode;
use dom::oscillatornode::OscillatorNode;
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::utils::{Reflector, reflect_dom_object};

#[dom_struct]
pub struct AudioContext {
    reflector_: Reflector,
    global: GlobalField,
    destination: Root<AudioDestinationNode>,
}

impl AudioContext {
    fn new_inherited(global: GlobalRef) -> AudioContext {
        AudioContext {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            destination: AudioDestinationNode::new(global),
        }
    }

    pub fn new(global: GlobalRef) -> Root<AudioContext> {
        reflect_dom_object(box AudioContext::new_inherited(global), global, AudioContextBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef)
                       -> Fallible<Root<AudioContext>> {
        Ok(AudioContext::new(global))
    }

}

impl<'a> AudioContextMethods for &'a AudioContext {

    fn Destination(self) -> Root<AudioDestinationNode> {
        Root::from_ref(&self.destination)
    }

    fn SampleRate(self) -> Finite<f32> {
        Finite::wrap(0f32)
    }

    fn CurrentTime(self) -> Finite<f64> {
        Finite::wrap(0f64)
    }

    fn CreateOscillator(self) -> Root<OscillatorNode> {
        OscillatorNode::new(self.global.root().r())
    }

}
