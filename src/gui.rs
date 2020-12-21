use std::borrow::Cow;
use std::sync::Mutex;
use std::thread;
use iced::{ 
    Application, Column, Text, Settings, Element, Container, Length, Rule, Row, Align, Space, ProgressBar, Command, Subscription
};
use iced::widget::{
    slider::{ self, Slider }, 
    pick_list::{ self, PickList }, 
    text_input::{ self, TextInput },
    scrollable::{ self, Scrollable },
    button::{ self, Button }
};
use iced::time;
use iced::executor;
use iced;

use crate::word_list::SearchType;
use crate::request::{ self, WordSearchRequest };
use crate::pdf;

const TITLE_SIZE: u16 = 40;
const TITLE_2_SIZE: u16 = 30;

lazy_static! {
    static ref ALL_GEN_TYPES: Vec<String> = vec![
        String::from("Related to [Word]"),
        String::from("Rhymes with [Word]"),
        String::from("Ends with [Letters]"),
        String::from("Sounds like [Word]"),
        String::from("Comes before [Word]"),
        String::from("Follows [Word]")
    ];

    static ref ALL_FORMATS: Vec<String> = vec![
        String::from("Letter"),
        String::from("Half Letter"),
        String::from("DINA4"),
        String::from("DINA5")
    ];

    static ref GEN_STATUS: Mutex<GenStatus> = {
        Mutex::new(GenStatus::InProgress)
    };
}

pub fn run() -> iced::Result {
    Gui::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })?;
    Ok(())
}

enum ProgressState {
    Creating,
    Generating
}

struct Gui {
    //Settings
    letter_count_sl: slider::State,
    letter_count: u8,
    word_count_sl: slider::State,
    word_count: u8,
    page_format_pl: pick_list::State<String>,
    page_format: &'static String,
    save_dir_in: text_input::State,
    save_dir: String,
    gen_button: button::State,
    //Word Search Fields
    word_search_list_scroll: scrollable::State,
    word_search_list: Vec<WordSearchField>,
    //Generation State
    progress_state: ProgressState
}

#[derive(Clone)]
struct WordSearchField {
    base_word_in: text_input::State,
    base_word: String,
    search_type_pl: pick_list::State<String>,
    search_type: &'static String,
    new_button: button::State,
    field_type: WordSearchFieldType,
    index: usize
}

#[derive(Clone)]
enum WordSearchFieldType {
    New,
    Input
}

#[derive(Debug, Clone)]
enum Message {
    StartGenerate,
    AddWordSearch,
    WordSearchFieldString((usize, String)),
    WordSearchFieldPickList((usize, String)),
    SettingsLetter(u8),
    SettingsWordNum(u8),
    SettingsPageFormat(String),
    SettingsSaveDir(String),
    Refresh
}

impl Application for Gui {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let gui = Gui {
            letter_count_sl: slider::State::new(),
            letter_count: 11,
            word_count_sl: slider::State::new(),
            word_count: 15,
            page_format_pl: pick_list::State::default(),
            page_format: &ALL_FORMATS[0],
            save_dir_in: text_input::State::new(),
            save_dir: String::from("TODO"),

            gen_button: button::State::new(),
            word_search_list: vec![
                WordSearchField {
                    base_word_in: text_input::State::new(),
                    base_word: String::from(""),
                    search_type_pl: pick_list::State::default(),
                    search_type: &ALL_GEN_TYPES[0],
                    new_button: button::State::new(),
                    field_type: WordSearchFieldType::New,
                    index: 0
                }
            ],
            word_search_list_scroll: scrollable::State::new(),

            progress_state: ProgressState::Creating
        };

        (gui, Command::none())
    }

    fn title(&self) -> String {
        String::from("Word Search Generator")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::AddWordSearch => {
                let len = self.word_search_list.len();
                let last_index = self.word_search_list[len - 1].index;
                self.word_search_list[len - 1].field_type = WordSearchFieldType::Input;

                self.word_search_list.push(WordSearchField {
                    base_word_in: text_input::State::new(),
                    base_word: String::from(""),
                    search_type_pl: pick_list::State::default(),
                    search_type: &ALL_GEN_TYPES[0],
                    new_button: button::State::new(),
                    field_type: WordSearchFieldType::New,
                    index: last_index + 1
                });
            },
            Message::StartGenerate => {
                for i in 0..(self.word_search_list.len() - 1) {
                    if self.word_search_list[i].base_word == String::from("") {
                        println!("Error: The Base Word for Word Search {} is Missing", i + 1);
                        return Command::none();
                    }
                }
                if self.word_search_list.len() == 1 {
                    println!("Error: No Word Searches Have Been Created");
                    return Command::none();
                }

                self.progress_state = ProgressState::Generating;
                
                //Spawn generation thread
                let word_search_list = self.word_search_list.clone();
                let format = self.page_format.clone();
                let max_count = self.word_count as usize;
                let height = self.letter_count as usize;
                let width = self.letter_count as usize;
                thread::spawn(move || {
                    let mut requests: Vec<WordSearchRequest> = Vec::new();
                    for word_search in word_search_list {
                        if let WordSearchFieldType::Input = word_search.field_type {
                            let request = WordSearchRequest {
                                word: word_search.base_word.replace(" ", ""),
                                stype: get_search_type(word_search.search_type),
                                max_count: max_count,
                                height: height,
                                width: width
                            };

                            requests.push(request);
                        }
                    }

                    let (results, errors) = request::handle_requests(requests);
                    
                    let (width, height) = get_format(&format);
                    match pdf::create_pdf(results, width, height) {
                        Ok(val) => val,
                        Err(err) => println!("An error ocurred while generating the pdf\nError Message: {}", err)
                    };

                    set_gen_status(GenStatus::Done);
                });

            },
            Message::WordSearchFieldString((index, val)) => {
                self.word_search_list[index].base_word = val;
            },
            Message::WordSearchFieldPickList((index, val)) => {
                for type_name in &*ALL_GEN_TYPES {
                    if val == *type_name {
                        self.word_search_list[index].search_type = &type_name;
                    }
                }
            },
            Message::SettingsLetter(val) => {
                self.letter_count = val;
            },
            Message::SettingsWordNum(val) => {
                self.word_count = val;
            },
            Message::SettingsPageFormat(val) => {
                for format in &*ALL_FORMATS {
                    if val == *format {
                        self.page_format = &format;
                    }
                }
            },
            Message::SettingsSaveDir(val) => {
                self.save_dir = val;
            },
            Message::Refresh => {
                match get_gen_status() {
                    GenStatus::InProgress => (),
                    GenStatus::Done => {
                        self.reset();
                        self.progress_state = ProgressState::Creating;
                    }
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(250))
            .map(|_| Message::Refresh)
    }

    fn view(&mut self) -> Element<Message> {
        match self.progress_state {
            ProgressState::Creating => self.draw_creating(),
            ProgressState::Generating => self.draw_generating()
        }
    }
}

impl Gui {
    fn draw_creating(&mut self) -> Element<Message> {
        //General
        let item_width = 200;

        //Create Word Search Element inside scroll
        let mut word_search_scroll = Scrollable::new(&mut self.word_search_list_scroll) //Scroll Element
        .width(Length::Fill)
        .height(Length::Fill)
        .scrollbar_width(10)
        .align_items(Align::Center)
        .spacing(10);

        word_search_scroll = word_search_scroll.push(Text::new("Word Searches to Generate:").size(TITLE_2_SIZE)); //Title

        for item in &mut self.word_search_list {
            word_search_scroll = word_search_scroll.push(Rule::horizontal(50)); //Bar to divide elements
            match item.field_type {
                WordSearchFieldType::Input => { //Elements that accept input
                    let item_index = item.index;
                    let word_field = Row::with_children( //Element to accept the base word for generation
                        vec![
                            Text::new("Base Word:").into(),
                            TextInput::new(&mut item.base_word_in, "Type in Base Word Here", &item.base_word, move |val| {
                                Message::WordSearchFieldString((item_index, val))
                            }).width(Length::Units(item_width)).into()
                        ]
                    ).spacing(15);

                    let item_index = item.index;
                    let type_field = Row::with_children( //Element to accept the type of word search for generation
                        vec![
                            Text::new("Word Search Type:").into(),
                            PickList::new(
                                &mut item.search_type_pl,
                                Cow::from(&*ALL_GEN_TYPES),
                                Some(item.search_type.to_string()),
                                move |val| {
                                    Message::WordSearchFieldPickList((item_index, val))
                                }
                            ).width(Length::Shrink).into()
                        ]
                    ).spacing(15);

                    word_search_scroll = word_search_scroll.push(word_field)
                    .push(type_field)
                },
                WordSearchFieldType::New => { //Element for creating new input elements
                    word_search_scroll = word_search_scroll.push(
                        Button::new(&mut item.new_button, Text::new("Add a New Word Search"))
                        .on_press(Message::AddWordSearch)
                    );
                }
            }
        }

        //Create Settings Element
        let settings_spacing = 50;
        let settings_mini_spacing = 5;

        let mut settings_col = Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Align::Center)
        .push(Text::new("Word Search Settings:").size(TITLE_2_SIZE))
        .push(Space::with_height(Length::Units(settings_spacing)));

        settings_col = settings_col.push(Text::new("Number of Letters per Row (8 to 14):")) //Letter count slider
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            Slider::new(
                &mut self.letter_count_sl,
                (8 as u8)..=14,
                self.letter_count,
                Message::SettingsLetter
            ).width(Length::Units(item_width))
        ).push(Space::with_height(Length::Units(settings_spacing)));

        settings_col = settings_col.push(Text::new("Maximum Number of Words in List (10 to 20):")) //Word count slider
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            Slider::new(
                &mut self.word_count_sl,
                (10 as u8)..=20,
                self.word_count,
                Message::SettingsWordNum
            ).width(Length::Units(item_width))
        ).push(Space::with_height(Length::Units(settings_spacing)));

        settings_col = settings_col.push(Text::new("Page Format:")) //Page format list
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(PickList::new(
            &mut self.page_format_pl,
            Cow::from(&*ALL_FORMATS),
            Some(self.page_format.to_string()),
            Message::SettingsPageFormat
        ))
        .push(Space::with_height(Length::Units(settings_spacing)));

        settings_col = settings_col.push(Text::new("Save to:")) //Save Directory
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            TextInput::new(
                &mut self.save_dir_in,
                "Enter a Directory",
                &self.save_dir,
                Message::SettingsSaveDir
            ).width(Length::Units(item_width))
        ).push(Space::with_height(Length::Units(80)));

        settings_col = settings_col.push( //Generate Button
            Button::new(
                &mut self.gen_button,
                Text::new("Generate Word Searches")
            ).on_press(Message::StartGenerate)
        );


        //Combine Both Elements
        let content = Row::with_children(
            vec![ word_search_scroll.into(), settings_col.into() ]
        );
        let title = Text::new("Word Search Generator").size(TITLE_SIZE);

        let whole_display = Column::new()
        .align_items(Align::Center)
        .push(Space::with_height(Length::Units(30)))
        .push(title)
        .push(Space::with_height(Length::Units(40)))
        .push(content);

        Container::new(whole_display)
        .center_x()
        .into()
    }

    fn draw_generating(&mut self) -> Element<Message> {
        let mut col = Column::new()
        .height(Length::Fill)
        .width(Length::Fill)
        .align_items(Align::Center);

        let progress;
        let text;
        if !request::get_finished() {
            progress = request::get_progress();
            text = Text::new("Generating Word Search...").size(75);
        }
        else {
            progress = pdf::get_progress();
            text = Text::new("Creating Pdf...").size(75);
        }

        col = col.push(text)
        .push(
            ProgressBar::new(0.0..=100.0, progress * 100.0)
        );

        Container::new(col)
        .center_x()
        .center_y()
        .into()
    }

    fn reset(&mut self) {
        self.word_search_list = vec![
            WordSearchField {
                base_word_in: text_input::State::new(),
                base_word: String::from(""),
                search_type_pl: pick_list::State::default(),
                search_type: &ALL_GEN_TYPES[0],
                new_button: button::State::new(),
                field_type: WordSearchFieldType::New,
                index: 0
            }
        ];
    }
}

fn get_search_type(stype: &str) -> SearchType {
    match stype {
        "Related to [Word]" => SearchType::RelatedTo,
        "Rhymes with [Word]" => SearchType::RhymesWith,
        "Ends with [Letters]" => SearchType::EndsWith,
        "Sounds like [Word]" => SearchType::SoundsLike,
        "Comes before [Word]" => SearchType::BlankWord,
        _ => SearchType::WordBlank //Follows [Word]
    }
}

fn get_format(format: &str) -> (f32, f32) {
    match format {
        "Letter" => (612.0, 792.0),
        "Half Letter" => (396.0, 612.0),
        "DINA4" => (595.0, 842.0),
        _ => (420.0, 595.0) //DINA5
    }
}

#[derive(Clone)]
enum GenStatus {
    InProgress,
    Done
}

fn get_gen_status() -> GenStatus {
    match GEN_STATUS.lock() {
        Ok(val) => val.clone(),
        Err(_) => GenStatus::InProgress
    }
}

fn set_gen_status(status: GenStatus) {
    match GEN_STATUS.lock() {
        Ok(mut val) => *val = status.clone(),
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(100));
            set_gen_status(status.clone())
        }
    }
}