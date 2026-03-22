/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::objc::{id, HostObject, SelectorMap};
use crate::Environment;

pub struct NSNumber {
    pub value: u32,
}

impl HostObject for NSNumber {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn numberWithUnsignedInteger(env: &mut Environment, _class: id, value: u32) -> id {
    let obj = NSNumber { value };
    env.objc.register_host_object(Box::new(obj), &mut env.mem)
}

fn unsignedIntegerValue(env: &mut Environment, this: id) -> u32 {
    let host_obj = env.objc.get_host_object(this).expect("Invalid NSNumber instance");
    let ns_num = host_obj.as_any().downcast_ref::<NSNumber>().expect("Not an NSNumber");
    ns_num.value
}

pub fn register_class(env: &mut Environment, selectors: &mut SelectorMap) {
    crate::objc::objc_class!(
        env,
        selectors,
        "NSNumber",
        metaclass_methods: {
            selector!(numberWithUnsignedInteger:) => numberWithUnsignedInteger,
        },
        instance_methods: {
            selector!(unsignedIntegerValue) => unsignedIntegerValue,
            selector!(intValue) => unsignedIntegerValue,
        }
    );
}

