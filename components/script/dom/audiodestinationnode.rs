/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::AudioDestinationNodeBinding;
use dom::bindings::codegen::Bindings::AudioDestinationNodeBinding::AudioDestinationNodeMethods;
use dom::bindings::codegen::InheritTypes::AudioDestinationNodeDerived;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::eventtarget::{EventTarget};

#[dom_struct]
pub struct AudioDestinationNode {
    reflector_: Reflector,
    max_channel_count: u32,
}

impl AudioDestinationNodeDerived for EventTarget {
    fn is_audiodestinationnode(&self) -> bool {
        true
    }
}

impl AudioDestinationNode {
    fn new_inherited() -> AudioDestinationNode {
        AudioDestinationNode {
            reflector_: Reflector::new(),
            max_channel_count: 0u32,
        }
    }

    pub fn new(global: GlobalRef) -> Root<AudioDestinationNode> {
        reflect_dom_object(box AudioDestinationNode::new_inherited(), global, AudioDestinationNodeBinding::Wrap)
    }
}


impl<'a> AudioDestinationNodeMethods for &'a AudioDestinationNode {

    fn MaxChannelCount(self) -> u32 {
      self.max_channel_count
    }

}
