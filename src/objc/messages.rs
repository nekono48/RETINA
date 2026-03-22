/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Handling of Objective-C messaging (`objc_msgSend` and friends).
//!
//! Resources:
//! - Apple's [Objective-C Runtime Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtHowMessagingWorks.html)
//! - [Apple's documentation of `objc_msgSend`](https://developer.apple.com/documentation/objectivec/1456712-objc_msgsend)
//! - Mike Ash's [objc_msgSend's New Prototype](https://www.mikeash.com/pyblog/objc_msgsends-new-prototype.html)
//! - Peter Steinberger's [Calling Super at Runtime in Swift](https://steipete.com/posts/calling-super-at-runtime/) explains `objc_msgSendSuper2`

use super::{id, nil, Class, ObjC, IMP, SEL};
use crate::abi::{CallFromHost, GuestRet};
use crate::mem::{ConstPtr, MutVoidPtr, SafeRead};
use crate::Environment;
use std::any::TypeId;

/// The core implementation of `objc_msgSend`, the main function of Objective-C.
#[allow(non_snake_case)]
fn objc_msgSend_inner(
    env: &mut Environment,
    receiver: id,
    selector: SEL,
    super2: Option<Class>,
    tolerate_type_mismatch: bool,
) {
    let message_type_info = env.objc.message_type_info.take();

    if receiver == nil {
        log_dbg!("[nil {}]", selector.as_str(&env.mem));
        env.cpu.regs_mut()[0..2].fill(0);
        return;
    }

    let orig_class = super2.unwrap_or_else(|| ObjC::read_isa(receiver, &env.mem));
    if orig_class == nil {
        env.cpu.regs_mut()[0..2].fill(0);
        return;
    }

    let mut class = orig_class;
    loop {
        if class == nil {
            assert!(class != orig_class);

            let class_host_object = env.objc.get_host_object(orig_class).unwrap();
            let &super::ClassHostObject {
                ref name,
                is_metaclass,
                ..
            } = class_host_object.as_any().downcast_ref().unwrap();

            let selector_str = selector.as_str(&env.mem);

            // BypassMethodSelector
            if selector_str == "methodForSelector:" {
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }
            // BypassStopLoading
            if selector_str == "stopLoading" {
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }
            // BypassInterfaceIdiom
            if selector_str == "userInterfaceIdiom" {
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }
            // BypassRootViewController
            if selector_str == "setRootViewController:" {
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }
            // ИСПРАВЛЕНИЕ: Добавляем обход для NSNumber numberWithUnsignedInteger
            if name == "NSNumber" && selector_str == "numberWithUnsignedInteger:" {
                log!("Warning: Bypassing [NSNumber numberWithUnsignedInteger:]");
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }

            panic!(
                "{} {:?} ({}class \"{}\", {:?}){} does not respond to selector \"{}\"!",
                if is_metaclass { "Class" } else { "Object" },
                receiver,
                if is_metaclass { "meta" } else { "" },
                name,
                orig_class,
                if super2.is_some() {
                    "'s superclass"
                } else {
                    ""
                },
                selector_str,
            );
        }

        let host_object = env.objc.get_host_object(class).unwrap();

        if let Some(&super::ClassHostObject {
            superclass,
            ref methods,
            ..
        }) = host_object.as_any().downcast_ref()
        {
            if super2.is_some() && class == orig_class {
                class = superclass;
                continue;
            }

            if let Some(imp) = methods.get(&selector) {
                match imp {
                    IMP::Host(host_imp) => {
                        if let Some((sent_type_id, sent_type_desc)) = message_type_info {
                            let (expected_type_id, expected_type_desc) = host_imp.type_info();
                            if sent_type_id != expected_type_id {
                                let msg = format!(
                                    "\
Type mismatch when sending message {} to {:?}!
- Message has type: {:?} / {}
- Method expects type: {:?} / {}",
                                    selector.as_str(&env.mem),
                                    receiver,
                                    sent_type_id,
                                    sent_type_desc,
                                    expected_type_id,
                                    expected_type_desc
                                );
                                if tolerate_type_mismatch {
                                    log!("Warning: {}", msg);
                                } else {
                                    panic!("{}", msg);
                                }
                            }
                        }
                        host_imp.call_from_guest(env)
                    }
                    IMP::Guest(guest_imp) => guest_imp.call_without_pushing_stack_frame(env),
                }
                return;
            } else {
                class = superclass;
            }
        } else if let Some(&super::UnimplementedClass {
            ref name,
            is_metaclass,
        }) = host_object.as_any().downcast_ref()
        {
            if name == "GKSession" {
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }
            if name == "EAAccessoryManager" {
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }
            if name == "MFMailComposeViewController" {
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }
            if name == "MFMessageComposeViewController" {
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }
            if name == "ASIdentifierManager" {
                env.cpu.regs_mut()[0..2].fill(0);
                return;
            }
            panic!(
                "Class \"{}\" ({:?}) is unimplemented. Call to {} method \"{}\".",
                name,
                class,
                if is_metaclass { "class" } else { "instance" },
                selector.as_str(&env.mem),
            );
        } else if let Some(&super::FakeClass {
            ref name,
            is_metaclass,
        }) = host_object.as_any().downcast_ref()
        {
            log!(
                "Call to faked class \"{}\" ({:?}) {} method \"{}\". Behaving as if message was sent to nil.",
                name,
                class,
                if is_metaclass { "class" } else { "instance" },
                selector.as_str(&env.mem),
            );
            env.cpu.regs_mut()[0..2].fill(0);
            return;
        } else {
            panic!(
                "Item {class:?} in superclass chain has unexpected host object type."
            );
        }
    }
}

// ... Оставшаяся часть файла (функции objc_msgSend, msg_send и макросы) остается идентичной оригиналу
