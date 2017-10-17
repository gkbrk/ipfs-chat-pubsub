extern crate gtk;
extern crate ipfsapi;
#[macro_use] extern crate error_chain;

use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;

use ipfsapi::IpfsApi;
use ipfsapi::pubsub::PubSubMessage;
use gtk::prelude::*;
use gtk::{Orientation, TextView, Entry, Button};

error_chain! {
    foreign_links {
    }
}

fn get_messages() -> Receiver<PubSubMessage> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        loop {
            let api = IpfsApi::new("127.0.0.1", 5001);

            if let Ok(messages) = api.pubsub_subscribe("chat") {
                for message in messages {
                    tx.send(message);
                }
            }
        }
    });

    return rx;
}

fn send_message(message: &str) {
    let message = message.to_string();
    thread::spawn(move || {
        let api = IpfsApi::new("127.0.0.1", 5001);
        api.pubsub_publish("chat", &message);
    });
}

#[derive(Clone)]
struct ChatWindow {
    chat_history: TextView,
    message_box: Entry,
    send_button: Button
}

impl ChatWindow {
    fn new() -> ChatWindow {
        ChatWindow {
            chat_history: TextView::new(),
            message_box: Entry::new(),
            send_button: Button::new_with_label("Send!")
        }
    }

    fn build_layout(&self) {
        
        let window = gtk::Window::new(gtk::WindowType::Toplevel);

        window.set_title("IPFS Chat");

        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(750, 400);

        let layout = gtk::Box::new(Orientation::Vertical, 3);
        let message_composer = gtk::Box::new(Orientation::Horizontal, 3);

        self.chat_history.set_editable(false);
        self.chat_history.set_cursor_visible(false);
        self.chat_history.set_size_request(-1, 300);

        message_composer.pack_start(&self.message_box, true, true, 1);
        message_composer.pack_start(&self.send_button, false, false, 1);

        layout.pack_start(&self.chat_history, true, true, 1);
        layout.pack_start(&message_composer, false, false, 1);

        let hist_clone = self.chat_history.clone();
        
        let incoming_messages = get_messages();

        let cl = Rc::new(RefCell::new(self.clone()));
        gtk::timeout_add(100, move || {
            while let Ok(message) = incoming_messages.try_recv() {
                if let Some(message) = message.data() {
                    let message = String::from_utf8(message).unwrap_or("Corrupted message".into());
                    cl.borrow().history_write_line(&format!("<message> {}", message));
                }
            }
            Continue(true)
        });

        let cl = Rc::new(RefCell::new(self.clone()));
        self.message_box.connect_activate(move |_| {
                cl.borrow().send_button_pressed();
        });

        let cl = Rc::new(RefCell::new(self.clone()));
        self.send_button.connect_clicked(move |_| {
                cl.borrow().send_button_pressed();
        });

        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        window.add(&layout);

        window.show_all();
        
    }

    fn history_write_line(&self, line: &str) {
        match self.chat_history.get_buffer() {
            Some(buffer) => {
                buffer.insert_at_cursor(line);
                buffer.insert_at_cursor("\n");
            },
            None => {println!("Cannot print line");}
        }
    }

    fn send_button_pressed(&self) {
        let text = self.message_box.get_text().unwrap();
        self.message_box.set_text("");
        if let Some(c) = text.chars().nth(0) {
            if c == '/' {
                let parts: Vec<&str> = text[1..].split_whitespace().collect();
                self.handle_command(&parts[0], parts[1..].to_vec());
            } else {
                send_message(&text);
            }
        }
    }

    fn handle_command(&self, command: &str, arguments: Vec<&str>) {
        match command {
            "quit" => {
                gtk::main_quit();
            },
            cmd => {
                self.history_write_line(&format!("Unknown command: {}", cmd));
            }
        }
    }
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK");
        return;
    }

    let app = ChatWindow::new();
    app.build_layout();
    gtk::main();
}
