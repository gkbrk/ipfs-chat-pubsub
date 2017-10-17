extern crate gtk;
extern crate ipfsapi;

use std::sync::mpsc::{channel, Receiver};
use std::thread;

use ipfsapi::IpfsApi;
use ipfsapi::pubsub::PubSubMessage;
use gtk::prelude::*;
use gtk::{Orientation, TextView, Entry, Button};

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

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK");
        return;
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    window.set_title("IPFS Chat");

    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(750, 400);

    let layout = gtk::Box::new(Orientation::Vertical, 3);
    let message_composer = gtk::Box::new(Orientation::Horizontal, 3);

    let chat_history = TextView::new();
    chat_history.set_editable(false);
    chat_history.set_cursor_visible(false);
    chat_history.set_size_request(-1, 300);

    let message_box = Entry::new();
    let send_button = Button::new_with_label("Send!");
    message_composer.pack_start(&message_box, true, true, 1);
    message_composer.pack_start(&send_button, false, false, 1);

    layout.pack_start(&chat_history, true, true, 1);
    layout.pack_start(&message_composer, false, false, 1);

    let hist_clone = chat_history.clone();
    let incoming_messages = get_messages();

    gtk::timeout_add(100, move || {
        while let Ok(message) = incoming_messages.try_recv() {
            if let Some(message) = message.data() {
                let message = String::from_utf8(message).unwrap_or("Corrupted message".into());
                hist_clone.get_buffer().unwrap().insert_at_cursor(&format!("<message> {}\n", message));
            }
        }
        Continue(true)
    });

    let msg_clone = message_box.clone();
    let hist_clone = chat_history.clone();
    message_box.connect_activate(move |_| {
        let text = msg_clone.get_text().unwrap();
        msg_clone.set_text("");
        send_message(&text);
    });

    let msg_clone = message_box.clone();
    let hist_clone = chat_history.clone();
    send_button.connect_clicked(move |_| {
        let text = msg_clone.get_text().unwrap();
        msg_clone.set_text("");
        send_message(&text);
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.add(&layout);

    window.show_all();
    gtk::main();
}
