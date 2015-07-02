/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::audiocontext::AudioContext;
use dom::bindings::codegen::Bindings::AudioNodeBinding;
use dom::bindings::codegen::Bindings::AudioNodeBinding::AudioNodeMethods;

use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};

#[dom_struct]
pub struct AudioNode {
    reflector_: Reflector,
    context: JS<AudioContext>,
}

impl AudioNode {
    fn new_inherited(context: &AudioContext) -> AudioNode {
        AudioNode {
            reflector_: Reflector::new(),
            context: JS::from_ref(context),
        }
    }

    pub fn new(global: GlobalRef, context: &AudioContext) -> Root<AudioNode> {
        reflect_dom_object(box AudioNode::new_inherited(context), global, AudioNodeBinding::Wrap)
    }
}

impl<'a> AudioNodeMethods for &'a AudioNode {

    fn Connect(self, destination: &AudioNode, output: u32, input: u32) -> Fallible<()> {
      return Ok(())
    }

    fn Disconnect(self, output: u32) -> Fallible<()> {
      return Ok(())
    }

    fn Context(self) -> Root<AudioContext> {
      return self.context.root();
    }

}


