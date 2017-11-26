extern crate pulldown_cmark;
extern crate orbtk;
extern crate orbfont;
extern crate orbclient;

use orbtk::point::Point;
use std::ops::Add;
use std::{cmp, env, str};
use std::sync::mpsc::{channel, Receiver};
use orbfont::Font;
use orbclient::{Color, Renderer, Window, WindowFlag, EventOption};


#[derive(Copy, Clone, Debug)]
pub struct Properties {
    pub font_size: Option<u32>,
    pub strong: Option<bool>,
    pub italic: Option<bool>,
    pub new_line: bool,
    pub margin_top: Option<i32>,
    pub margin_bottom: Option<i32>,
}

enum GuideCommand {
}

impl Properties {
    pub fn new() -> Self {
        Properties {
            font_size: None,
            strong: None,
            italic: None,
            new_line: false,
            margin_top: None,
            margin_bottom: None,
        }
    }
}

#[derive(Clone, Debug)]
struct Node {
    text: String,
    properties: Properties
}

impl Node {

    pub fn new(text: String, properties: Properties) -> Self {
        Node {
            text: text,
            properties: properties,
        }
    }

}

pub struct Block<'a> {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: Color,
    text: orbfont::Text<'a>,
}

impl<'a> Block<'a> {
    fn draw(&self, window:&mut Window) {
        self.text.draw(window, self.x, self.y, Color::rgb(0, 0, 0));
    }
}

pub struct Guide<'a> {
    window: Window,
    window_width: u32,
    window_height: u32,
    rx: Receiver<GuideCommand>,
    nodes: Vec<Node>,
    blocks: Vec<Block<'a>>,
    font_normal: &'a Font,
    font_bold: &'a Font,
    file: String,
}

impl<'a> Guide<'a> {
    pub fn new(file: &str, font_normal: &'a Font, font_bold: &'a Font) -> Self {
        let (tx, rx) = channel();

        let window_width = 500;
        let window_height = 500;
        let window = Window::new_flags(-1, -1, window_width, window_height, "Guide", &[WindowFlag::Resizable]).unwrap();

        Guide {
            window: window,
            window_width: window_width,
            window_height: window_height,
            rx: rx,
            nodes: Vec::new(),
            blocks: Vec::new(),
            font_normal: font_normal,
            font_bold: font_bold,
            file: String::from(file),
        }
    }

    pub fn render(&mut self) {
        println!("rendering {:?}", self.nodes);
        
   
        let mut pos = Point::new(0,0);
        let mut previous_height = 0;

        for (idx, mut node) in self.nodes.iter().enumerate() {

            let trimmed_left = node.text.trim_left();
            let left_margin = node.text.len() as i32 - trimmed_left.len() as i32;
            let trimmed_right = trimmed_left.trim_right();
            let right_margin = trimmed_left.len() as i32 - trimmed_right.len() as i32;

            let font_height = node.properties.font_size.unwrap_or(20) as f32;
            let mut new_line = node.properties.new_line;

            pos.x += left_margin * 8;

            for (word_i, word) in trimmed_right.split(' ').enumerate() {
                if word_i > 0 {
                    pos.x += 8;
                }
                let text = match node.properties.strong {
                    Some(true) => self.font_bold.render(word, font_height),
                    _ => self.font_normal.render(word, font_height)
                };

                let w = text.width() as i32;
                let h = text.height() as i32;

                if new_line {
                    new_line = false;
                    pos.x = 0;

                    if idx != 0 {
                        pos.y += previous_height as i32;
                    }
                }

                if pos.x + w >= self.window_width as i32 && pos.x > 0 {
                    pos.x = 0;
                    pos.y += h as i32;
                }

                if word_i == 0 {
                    if let Some(margin) = node.properties.margin_top {
                        pos.y += margin;
                    }
                }

                self.blocks.push(Block {
                    x: pos.x,
                    y: pos.y,
                    width: w,
                    height: h,
                    color: Color::rgb(0, 0, 0),
                    text: text
                });

                previous_height = h;
                pos.x += w;
            }

            pos.x += right_margin * 8;
        }

    }

    pub fn parse(&mut self) {
        let small = r##"# This is header 1

Lorem ipsum __strong__ text that is too long and breaks __line__
        
Need to find out what to do from now on

# This is __bold__ headerx 1

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nemo nostrum istius generis asotos iucunde putat vivere.

## This is header 2

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nemo nostrum istius generis asotos iucunde putat vivere.
"##;
        use std::fs::File;
        use std::io::prelude::*;
        let mut contents = String::new();
        {
            let file_name = &mut self.file;
            let mut file = File::open(file_name).expect("file not found");
            file.read_to_string(&mut contents).expect("something went wrong reading the file");
        }

        use pulldown_cmark::{Parser, Event, Options, Tag};

        let opts = Options::empty();

        let mut property_list: Vec<Properties> = Vec::new();

        let parser = Parser::new_ext(&contents, opts);


        for event in parser {
            match event {
                Event::Start(Tag::Paragraph) => {
                    println!("Pushing paragraph");
                    let mut property = match property_list.pop() {
                        Some(tmp) => {
                            property_list.push(tmp);
                            tmp.clone()
                        }
                        None => Properties::new()
                    };
                    property.new_line = true;
                    property_list.push(property);
                }
                Event::Start(Tag::Header(size)) => {
                    println!("Pushing header");
                    let mut property = Properties::new();                    
                    property.new_line = true;
                    property.font_size = match size {
                        1 => Some(40),
                        2 => Some(30),
                        _ => Some(20),
                    };
                    property.margin_top = match size {
                        1 => Some(40),
                        2 => Some(40),
                        _ => Some(40),
                    };
                    property.margin_bottom = property.margin_top;
                    
                    property_list.push(property);
                }
                Event::Start(Tag::Strong) | Event::Start(Tag::Emphasis) => {
                    println!("Pushing Strong");
                    let mut property = match property_list.pop() {
                        Some(mut tmp) => {
                            let new = tmp.clone();
                            tmp.new_line = false;
                            tmp.margin_top = None;
                            property_list.push(tmp);
                            new
                        }
                        None => Properties::new()
                    };
                    property.strong = Some(true);
                    property.new_line = false;
                    
                    property_list.push(property);
                }
                Event::Text(text) => {
                    println!("Pushing Text {}", text);
                    let properties = match property_list.pop() {
                        Some(mut tmp) => {
                            let new = tmp.clone();
                            tmp.new_line = false;
                            tmp.margin_top = None;
                            property_list.push(tmp);
                            new
                        }
                        None => Properties::new()
                    };
                    self.nodes.push(Node::new(String::from(text), properties));

                }
                Event::Start(x) => {
                    println!("Pushing unknown element {:?}", x);

                }
                Event::End(_) => {
                    property_list.pop();
                    println!("End of element");

                }
                _ => {}
            }
        }
        self.render();
    }

    

    pub fn redraw(&mut self) {
        self.parse();
    }

    pub fn exec(&mut self) {
        self.redraw();
        let mut redraw = true;

        loop {

            if redraw {
                redraw = false;

                {
                    self.window.set(Color::rgb(255, 255, 255));

                    for block in self.blocks.iter() {
                        block.draw(&mut self.window);
                    }

                    self.window.sync();
                }
            }

            for event in self.window.events() {
                match event.to_option() {
                    EventOption::Resize(_) => {
                        redraw = true;
                    },
                    EventOption::Quit(_) => return,
                    _ => ()
                }
            }
        }
    }
}

fn main() {
    match Font::find(None, None, None) {
    Ok(font_normal) => {
        match Font::find(None, None, Some("Bold")) {
            Ok(font_bold) => {
                Guide::new(&env::args().nth(1).unwrap_or("examples/loremipsum.md".to_string()), &font_normal, &font_bold).exec()
            },
            Err(_) => {println!("ERROR GETTING FONT")}
            }

        },
        Err(_) => {println!("ERROR GETTING FONT")}
    };

}
