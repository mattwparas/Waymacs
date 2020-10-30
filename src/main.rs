#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else
)]
mod document;
mod editor;
mod filetype;
mod highlighting;
mod light_worker;
mod row;
mod terminal;
mod utilities;
pub use document::Document;
use editor::Editor;
pub use editor::Position;
pub use editor::SearchDirection;
pub use filetype::FileType;
pub use filetype::HighlightingOptions;
pub use row::Row;
pub use terminal::Terminal;

#[macro_use]
extern crate lazy_static;

extern crate inflector;
// extern crate notify_rust;
extern crate webbrowser;

extern crate cocoa;
#[macro_use]
extern crate objc;

use std::cell::RefCell;
use std::rc::Rc;

extern crate steel;
#[macro_use]
extern crate steel_derive;

use steel::unwrap;

use std::any::Any;
use steel::rerrs;
use steel::rvals::{self, CustomType, SteelVal};

// use steel::build_interpreter;
// use steel::build_repl;
use steel::build_vm;
// use steel::repl::repl_base;
use steel::rvals::StructFunctions;
use steel::vm::VirtualMachine;
use steel_derive::function;
use steel_derive::steel;

// use steel::Gc;

// use std::process;

// use std::rc::Rc;

use steel::SteelErr;

// use std::env::args;
// use std::fs;
// use steel::PRELUDE;

use std::convert::TryFrom;

use std::fmt::Write;

use steel::gc::Gc;

#[steel]
pub struct EditorWrapper(pub Rc<RefCell<Editor>>);

pub fn default_editor() -> EditorWrapper {
    EditorWrapper(Rc::new(RefCell::new(Editor::default())))
}

#[function]
pub fn current_buffer_as_string(this: EditorWrapper) -> String {
    this.0.borrow().current_buffer().into_iter().collect()
}

#[function]
pub fn current_buffer_as_list(this: EditorWrapper) -> Vec<String> {
    this.0
        .borrow()
        .current_buffer()
        .into_iter()
        .map(|x| x.into())
        .collect()
}

#[function]
pub fn insert_into_buffer(this: EditorWrapper, s: String) {
    this.0.borrow_mut().insert_at_current_position(s)
}

#[steel]
pub struct VecWrapper {
    inner: Vec<String>,
}

impl VecWrapper {
    pub fn get_inner(&self) -> Vec<String> {
        self.inner.clone()
    }
}

#[function]
pub fn list_to_buffer(this: EditorWrapper, lst: Vec<String>) {
    // panic!("get here");
    this.0.borrow_mut().write_rows_to_buffer(lst)
}

// use mac_notification_sys::*;
fn main() {
    // crate::utilities::show_alert();

    crate::light_worker::spawn_workers();
    // Editor::default().run();
    // crate::light_worker::join_all();

    let editor = default_editor();

    let mut vm = build_vm! {
        Structs => {
            VecWrapper
        }
        Functions => {
            "buffer->list'" => current_buffer_as_list,
            "buffer->string'"=> current_buffer_as_string,
            "insert!'" => insert_into_buffer,
            "list->buffer!'" => list_to_buffer,
        }
    };

    // bind the editor instance here
    vm.insert_binding("*editor*".to_string(), editor.clone().new_steel_val());

    let mut message = "".to_string();

    // Load in the standard library
    vm.parse_and_execute(steel::PRELUDE)
        .expect("error loading the prelude");

    vm.parse_and_execute(
        r#"
                    (define (buffer->list) (buffer->list' *editor*))
                    (define (buffer->string) (buffer->string' *editor*))
                    (define (insert! str) (insert!' *editor* str))
                    (define (list->buffer! lst) (list->buffer!' *editor* lst))
                    (define (all-upper!) (list->buffer! (map string-upcase (buffer->list))))
        "#,
    )
    .expect("Error loading the editor functions!");

    loop {
        let callback_result = { editor.0.borrow_mut().run(message) };

        match callback_result {
            Some(command) if command == "run" => {
                // let current_buffer = {  };
                let joined_buffer: String =
                    { editor.0.borrow().current_buffer().into_iter().collect() };

                let evaluated_buffer = vm.parse_and_execute(joined_buffer.as_str());

                match evaluated_buffer {
                    Ok(vals) => {
                        // let last = vals.last();
                        if let Some(last) = vals.last() {
                            message = last.to_string();
                        } else {
                            message = "error - no results returned".to_string()
                        }
                    }
                    Err(e) => message = e.to_string(),
                }
            }
            Some(command) if command != "" => match vm.parse_and_execute(command.as_str()) {
                Ok(vals) => {
                    // let last = vals.last();
                    if let Some(last) = vals.last() {
                        message = last.to_string();
                    } else {
                        message = "error - no results returned".to_string()
                    }
                }
                Err(e) => message = e.to_string(),
            },
            _ => break,
        }
    }
}
