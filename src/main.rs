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


enum GuideCommand {
}

#[derive(Copy, Clone, Debug)]
pub struct Properties {
    pub font_size: Option<u32>,
    pub strong: Option<bool>,
    pub italic: Option<bool>,
}

impl Properties {
    pub fn new() -> Self {
        Properties {
            font_size: None,
            strong: None,
            italic: None,
        }
    }
}


#[derive(Clone, Debug)]
struct RootNode {
    pub new_line: bool,
    pub margin_top: i32,
    pub margin_bottom: i32,
    pub nodes: Vec<Node>,
}

impl RootNode {
    pub fn new() -> Self {
        RootNode {
            new_line: false,
            margin_top: 0,
            margin_bottom: 0,
            nodes: Vec::new(),
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
    fn draw(&self, window: &mut Window, offset: Point) {
        let x = self.x - offset.x;
        let y = self.y - offset.y;
        if x + self.width > 0 && x < window.width() as i32 && y + self.height > 0 && y < window.height() as i32 {
            self.text.draw(window, x, y, Color::rgb(0, 0, 0));
        }
    }
}

pub struct Guide<'a> {
    window: Window,
    window_width: u32,
    window_height: u32,
    offset: Point,
    max_offset: Point,
    rx: Receiver<GuideCommand>,
    root_nodes: Vec<RootNode>,
    blocks: Vec<Block<'a>>,
    font_normal: &'a Font,
    font_bold: &'a Font,
    file: String,
}

impl<'a> Guide<'a> {
    pub fn new(file: &str, font_normal: &'a Font, font_bold: &'a Font) -> Self {
        let (tx, rx) = channel();

        let window_width = 800;
        let window_height = 900;
        let window = Window::new_flags(-1, -1, window_width, window_height, "Guide", &[WindowFlag::Resizable]).unwrap();

        Guide {
            window: window,
            window_width: window_width,
            window_height: window_height,
            offset: Point::new(0, 0),
            max_offset: Point::new(0, 0),
            rx: rx,
            root_nodes: Vec::new(),
            blocks: Vec::new(),
            font_normal: font_normal,
            font_bold: font_bold,
            file: String::from(file),
        }
    }

    pub fn render(&mut self) {
        println!("rendering {:?}", self.root_nodes);

        let mut pos = Point::new(0,0);
        let mut previous_height = 0;
        let mut previous_root_node = RootNode::new();

        for mut root_node in self.root_nodes.iter() {
            let mut new_line = root_node.new_line;

            pos.y += previous_root_node.margin_bottom;
            pos.y += root_node.margin_top;

            for (idx, mut node) in root_node.nodes.iter().enumerate() {

                let trimmed_left = node.text.trim_left();
                let left_margin = node.text.len() as i32 - trimmed_left.len() as i32;
                let trimmed_right = trimmed_left.trim_right();
                let right_margin = trimmed_left.len() as i32 - trimmed_right.len() as i32;

                let font_height = node.properties.font_size.unwrap_or(20) as f32;

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

                        pos.y += previous_height as i32;
                    }

                    if pos.x + w >= self.window_width as i32 && pos.x > 0 {
                        pos.x = 0;
                        pos.y += h as i32;
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
            previous_root_node = root_node.clone();
        }

        let mut max_offset = Point::new(0, 0);
        for block in self.blocks.iter() {
            if block.x + block.width > max_offset.x {
                max_offset.x = block.x + block.width;
            }
            if block.y + block.height > max_offset.y {
                max_offset.y = block.y + block.height;
            }
        }
        self.max_offset = max_offset;

    }

    pub fn parse(&mut self) {
        let small = r##"# This is header 1

Lorem ipsum __strong__ text that is too long and breaks __line__ at least once

## Header 2

Istud quidem, inquam, optime dicis, sed quaero nonne tibi faciendum idem sit nihil dicenti bonum, quod non rectum honestumque sit, reliquarum rerum discrimen omne tollenti.

"##;
        use std::fs::File;
        use std::io::prelude::*;
        let mut contents = String::new();
        {
            let mut file = File::open(&mut self.file).expect("file not found");
            file.read_to_string(&mut contents).expect("something went wrong reading the file");
        }

        use pulldown_cmark::{Parser, Event, Options, Tag};

        let opts = Options::empty();

        let mut property_list: Vec<Properties> = Vec::new();

        let parser = Parser::new_ext(&contents, opts);

        let mut tmp_root = RootNode::new();

        for event in parser {
            match event {
                Event::Start(Tag::Paragraph) => {
                    println!("Pushing paragraph");
                    tmp_root.new_line = true;
                    tmp_root.margin_top = 5;
                    tmp_root.margin_bottom = 10;
                    property_list.push(Properties::new());
                }
                Event::Start(Tag::Header(size)) => {
                    println!("Pushing header");
                    let mut property = Properties::new();
                    property.font_size = match size {
                        1 => {
                            property.strong = Some(true);
                            Some(40)
                        },
                        2 => Some(33),
                        _ => Some(25),
                    };
                    tmp_root.new_line = true;
                    tmp_root.margin_top = match size {
                        1 => 30,
                        2 => 20,
                        _ => 15,
                    };
                    tmp_root.margin_bottom = tmp_root.margin_top;

                    property_list.push(property);
                }
                Event::Start(Tag::Strong) | Event::Start(Tag::Emphasis) => {
                    println!("Pushing Strong");
                    
                    let mut property = match property_list.pop() {
                        Some(mut tmp) => {
                            property_list.push(tmp);
                            tmp.clone()
                        }
                        None => Properties::new()
                    };
                    property.strong = Some(true);                    
                    property_list.push(property);
                }
                Event::Text(text) => {
                    println!("Pushing Text {}", text);
                    let properties = match property_list.pop() {
                        Some(mut tmp) => {
                            property_list.push(tmp);
                            tmp.clone()
                        }
                        None => Properties::new()
                    };
                    tmp_root.nodes.push(Node::new(String::from(text), properties));

                }
                Event::Start(x) => {
                    println!("Pushing unknown element {:?}", x);

                }
                Event::End(_) => {
                    property_list.pop();
                    if property_list.len() == 0 {
                        self.root_nodes.push(tmp_root.clone());
                        tmp_root = RootNode::new();

                        println!("# PUSHING ROOT NODE")
                    }
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
                        block.draw(&mut self.window, self.offset);
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
                    EventOption::Scroll(scroll_event) => {
                        self.offset.x = cmp::max(0, cmp::min(cmp::max(0, self.max_offset.x - self.window_width as i32), self.offset.x - scroll_event.x * 48));
                        self.offset.y = cmp::max(0, cmp::min(cmp::max(0, self.max_offset.y - self.window_height as i32), self.offset.y - scroll_event.y * 48));

                        redraw = true;
                    },
                    _ => (),
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
                Guide::new(&env::args().nth(1).unwrap_or("examples/elements.md".to_string()), &font_normal, &font_bold).exec()
            },
            Err(_) => {println!("ERROR GETTING FONT")}
            }

        },
        Err(_) => {println!("ERROR GETTING FONT")}
    };

}
