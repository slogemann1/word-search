use std::borrow::Cow;
use std::sync::Mutex;
use std::thread;
use iced::{ 
    Application, Column, Text, Settings, Element, Container, Length, Rule, Row,
    Align, Space, ProgressBar, Command, Subscription, HorizontalAlignment
};
use iced::widget::{
    slider::{ self, Slider }, 
    pick_list::{ self, PickList }, 
    text_input::{ self, TextInput },
    scrollable::{ self, Scrollable },
    button::{ self, Button }
};
use iced::window;
use iced::time;
use iced::executor;
use iced;

use crate::word_list::SearchType;
use crate::request::{ self, WordSearchRequest };
use crate::config::{ self, Preferences };
use crate::pdf;
use crate::img;

mod styling;
use styling::Theme;

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

    static ref ALL_THEMES: Vec<String> = vec![
        String::from("Light"),
        String::from("Dark")
    ];

    static ref GEN_STATUS: Mutex<GenStatus> = {
        Mutex::new(GenStatus::InProgress)
    };

    static ref ERR_MSGS: Mutex<Vec<String>> = {
        Mutex::new(Vec::new())
    };
}

pub fn run() -> iced::Result {
    let window_settings = window::Settings {
        icon: img::get_icon(),
        ..window::Settings::default()
    };

    Gui::run(Settings {
        window: window_settings,
        antialiasing: true,
        ..Settings::default()
    })?;
    Ok(())
}

enum ProgressState {
    Creating,
    Generating,
    Finished,
    ChangingSettings,
}

struct Gui {
    //Word Search Settings
    letter_count_sl: slider::State,
    letter_count: u8,
    word_count_sl: slider::State,
    word_count: u8,
    page_format_pl: pick_list::State<String>,
    page_format: &'static String,
    save_dir_in: text_input::State,
    save_dir: String,
    gen_button: button::State,
    go_to_settings_button: button::State,
    //Word Search Fields
    word_search_list_scroll: scrollable::State,
    word_search_list: Vec<WordSearchField>,
    //Generation State
    progress_state: ProgressState,
    finished_button: button::State,
    //Errors
    err_msg: String,
    err: bool,
    //Theme
    theme: Theme,
    //Settings
    theme_sett_pl: pick_list::State<String>,
    theme_sett: &'static String,
    word_count_sett_in: text_input::State,
    word_count_sett: String,
    letter_count_sett_in: text_input::State,
    letter_count_sett: String,
    page_format_sett_pl: pick_list::State<String>,
    page_format_sett: &'static String,
    save_dir_sett_in: text_input::State,
    save_dir_sett: String,
    save_sett_button: button::State,
    return_button: button::State
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
    Letter(u8),
    WordNum(u8),
    PageFormat(String),
    SaveDir(String),
    Reset,
    Refresh,
    SaveSettings,
    GotoSettings,
    ReturnFromSettings,
    SettingsTheme(String),
    SettingsLetter(String),
    SettingsWordNum(String),
    SettingsPageFormat(String),
    SettingsSaveDir(String),
}

impl Application for Gui {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let gui = Gui::new_from_prefs(config::get_preferences());

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
                        self.err_msg = format!("Error: The Base Word for Word Search {} is Missing", i + 1);
                        self.err = true;
                        return Command::none();
                    }
                }
                if self.word_search_list.len() == 1 {
                    self.err_msg = format!("Error: No Word Searches Have Been Created");
                    self.err = true;
                    return Command::none();
                }
                if self.save_dir == "" {
                    self.err_msg = format!("Error: No Saving Directory Has Been Specified");
                    self.err = true;
                    return Command::none();
                }

                self.progress_state = ProgressState::Generating;
                
                //Spawn generation thread
                let word_search_list = self.word_search_list.clone();
                let format = self.page_format.clone();
                let save_dir = self.save_dir.clone();
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
                    set_err_msgs(errors);

                    let (width, height) = get_format(&format);
                    match pdf::create_pdf(results, width, height, &save_dir) {
                        Ok(val) => val,
                        Err(_) => ()
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
            Message::Letter(val) => {
                self.letter_count = val;
            },
            Message::WordNum(val) => {
                self.word_count = val;
            },
            Message::PageFormat(val) => {
                for format in &*ALL_FORMATS {
                    if val == *format {
                        self.page_format = &format;
                    }
                }
            },
            Message::SaveDir(val) => {
                self.save_dir = val;
            },
            Message::SettingsLetter(val) => {
                self.letter_count_sett = val;
            },
            Message::SettingsWordNum(val) => {
                self.word_count_sett = val;
            },
            Message::SettingsPageFormat(val) => {
                for format in &*ALL_FORMATS {
                    if val == *format {
                        self.page_format_sett = &format;
                    }
                }
            },
            Message::SettingsSaveDir(val) => {
                self.save_dir_sett = val;
            },
            Message::SettingsTheme(val) => {
                match val.as_str() {
                    "Light" => self.theme_sett = &ALL_THEMES[0],
                    "Dark" => self.theme_sett = &ALL_THEMES[1],
                    _ => self.theme_sett = &ALL_THEMES[0]
                }
            },
            Message::Refresh => {
                match get_gen_status() {
                    GenStatus::InProgress => (),
                    GenStatus::Done => {
                        set_gen_status(GenStatus::InProgress);
                        self.progress_state = ProgressState::Finished;
                    }
                }
            },
            Message::Reset => {
                self.progress_state = ProgressState::Creating;
                self.reset();
            },
            Message::GotoSettings => {
                self.progress_state = ProgressState::ChangingSettings;
            },
            Message::SaveSettings => {
                let theme = self.theme_sett.clone();
                let word_count = match self.word_count_sett.parse::<f64>() { //Floats so all numbers are numbers
                    Ok(val) => {
                        if val >= 10.0 && val <= 20.0 {
                            val as u8
                        }
                        else {
                            self.err = true;
                            self.err_msg = format!("Error: {} is outside the range of word counts", self.word_count_sett);
                            return Command::none();
                        }
                    },
                    Err(_) => {
                        self.err = true;
                        self.err_msg = format!("Error: {} is not a number", self.word_count_sett);
                        return Command::none();
                    }
                };
                let letter_count = match self.letter_count_sett.parse::<f64>() {
                    Ok(val) => {
                        if val >= 8.0 && val <= 14.0 {
                            val as u8
                        }
                        else {
                            self.err = true;
                            self.err_msg = format!("Error: {} is outside the range of letter counts", self.word_count_sett);
                            return Command::none();
                        }
                    },
                    Err(_) => {
                        self.err = true;
                        self.err_msg = format!("Error: {} is not a number", self.letter_count_sett);
                        return Command::none();
                    }
                };
                let format = self.page_format_sett.clone();
                let save_dir = self.save_dir_sett.clone();

                let prefs = Preferences {
                    theme: theme,
                    word_count: word_count,
                    letter_count: letter_count,
                    format: format,
                    save_directory: save_dir
                };

                if config::save_preferences(prefs.clone()) == false {
                    self.err = true;
                    self.err_msg = String::from("Error: Failed to save settings");
                    return Command::none()
                }

                //Update Gui
                let word_searches = self.word_search_list.clone();
                *self = Gui::new_from_prefs(prefs);
                self.word_search_list = word_searches;
                self.progress_state = ProgressState::Creating;
            },
            Message::ReturnFromSettings => {
                self.progress_state = ProgressState::Creating;
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
            ProgressState::Generating => self.draw_generating(),
            ProgressState::Finished => self.draw_finished(),
            ProgressState::ChangingSettings => self.draw_settings()
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
                            })
                            .width(Length::Units(item_width))
                            .style(self.theme.clone())
                            .into()
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
                            )
                            .width(Length::Shrink)
                            .style(self.theme.clone())
                            .into()
                        ]
                    ).spacing(15);

                    word_search_scroll = word_search_scroll.push(word_field)
                    .push(type_field)
                },
                WordSearchFieldType::New => { //Element for creating new input elements
                    word_search_scroll = word_search_scroll.push(
                        Button::new(&mut item.new_button, Text::new("Add a New Word Search"))
                        .on_press(Message::AddWordSearch)
                        .style(self.theme.clone())
                    );
                }
            }
        }

        //Create Word Search Settings Element
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
                Message::Letter
            ).width(Length::Units(item_width))
        ).push(Space::with_height(Length::Units(settings_spacing)));

        settings_col = settings_col.push(Text::new("Maximum Number of Words in List (10 to 20):")) //Word count slider
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            Slider::new(
                &mut self.word_count_sl,
                (10 as u8)..=20,
                self.word_count,
                Message::WordNum
            ).width(Length::Units(item_width))
        ).push(Space::with_height(Length::Units(settings_spacing)));

        settings_col = settings_col.push(Text::new("Page Format:")) //Page format list
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            PickList::new(
            &mut self.page_format_pl,
            Cow::from(&*ALL_FORMATS),
            Some(self.page_format.to_string()),
            Message::PageFormat
            )
            .style(self.theme.clone())
        )
        .push(Space::with_height(Length::Units(settings_spacing)));

        settings_col = settings_col.push(Text::new("Save to:")) //Save Directory
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            TextInput::new(
                &mut self.save_dir_in,
                "Enter a Directory",
                &self.save_dir,
                Message::SaveDir
            )
            .width(Length::Units(item_width))
            .style(self.theme.clone())
        ).push(Space::with_height(Length::Units(60)));

        settings_col = settings_col.push( //Generate Button
            Button::new(
                &mut self.gen_button,
                Text::new("Generate Word Searches").size(35)
            )
            .on_press(Message::StartGenerate)
            .style(self.theme.clone())
        );

        if self.err {
            settings_col = settings_col.push(Space::with_height(Length::Units(45)))
            .push(Text::new(&self.err_msg));
        }

        //Combine Both Elements
        let content = Row::with_children(
            vec![ word_search_scroll.into(), settings_col.into() ]
        );
        let title = Text::new("Word Search Generator").size(TITLE_SIZE);

        let settings_button_row = Row::with_children(
            vec![
                Space::with_width(Length::Fill)
                .into(),
                Button::new(&mut self.go_to_settings_button, Text::new("Settings").size(20))
                .on_press(Message::GotoSettings)
                .style(self.theme.clone())
                .width(Length::Shrink)
                .into(),
            ]
        );

        let whole_display = Column::new()
        .align_items(Align::Center)
        .push(settings_button_row)
        //.push(Space::with_height(Length::Units(30)))
        .push(title)
        .push(Space::with_height(Length::Units(40)))
        .push(content);

        Container::new(whole_display)
        .center_x()
        .style(self.theme.clone())
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

        col = col.push(Space::with_height(Length::Units(200)))
        .push(text)
        .push(
            ProgressBar::new(0.0..=100.0, progress * 100.0)
        )
        .push(Space::with_height(Length::Units(500)));

        Container::new(col)
        .center_x()
        .center_y()
        .style(self.theme.clone())
        .into()
    }

    fn draw_finished(&mut self) -> Element<Message> {
        let mut col = Column::new()
        .height(Length::Fill)
        .width(Length::Fill)
        .align_items(Align::Center);

        col = col.push(Space::with_height(Length::Units(200)))
        .push(
            Text::new("Finished Generating Word Searches!")
            .horizontal_alignment(HorizontalAlignment::Center)
            .size(75)
        )
        .push(Space::with_height(Length::Units(50)))
        .push(
            Button::new(&mut self.finished_button, Text::new("Click Here to Continue").size(50))
            .on_press(Message::Reset)
            .style(self.theme.clone())
        );

        let err_msgs = get_err_msgs();
        if err_msgs.len() > 0 {
            col = col.push(Space::with_height(Length::Units(50)))
            .push(Text::new("Error messages:").size(30));

            for msg in err_msgs {
                col = col.push(Space::with_height(Length::Units(10)))
                .push(Text::new(&msg).size(15));
            }
        }

        Container::new(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(self.theme.clone())
        .into()
    }

    fn draw_settings(&mut self) -> Element<Message> {
        let settings_spacing = 35;
        let settings_mini_spacing = 15;
        let item_width = Length::Units(310);

        let return_button_row = Row::with_children(
            vec![
                Space::with_width(Length::Fill)
                .into(),
                Button::new(&mut self.return_button, Text::new("Return").size(20))
                .on_press(Message::ReturnFromSettings)
                .style(self.theme.clone())
                .width(Length::Shrink)
                .into(),
            ]
        );

        let mut col = Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Align::Center)
        .push(return_button_row)
        .push(Text::new("Settings:").size(TITLE_SIZE))
        .push(Space::with_height(Length::Units(settings_spacing)));

        col = col.push(Text::new("Theme:")) //Theme List
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            PickList::new(
                &mut self.theme_sett_pl,
                Cow::from(&*ALL_THEMES),
                Some(self.theme_sett.to_string()),
                Message::SettingsTheme
            )
            .style(self.theme.clone())
        ).push(Space::with_height(Length::Units(settings_spacing)));
        
        col = col.push(Text::new("Number of Letters per Row (8 to 14):")) //Letter count field
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            TextInput::new(
                &mut self.letter_count_sett_in,
                "Enter Default Letter Count Here",
                &mut self.letter_count_sett,
                Message::SettingsLetter
            )
            .width(item_width)
            .style(self.theme.clone())
        ).push(Space::with_height(Length::Units(settings_spacing)));

        col = col.push(Text::new("Maximum Number of Words in List (10 to 20):")) //Word count field
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            TextInput::new(
                &mut self.word_count_sett_in,
                "Enter Default Word Count Here",
                &mut self.word_count_sett,
                Message::SettingsWordNum
            )
            .width(item_width)
            .style(self.theme.clone())
        ).push(Space::with_height(Length::Units(settings_spacing)));

        col = col.push(Text::new("Page Format:")) //Page format list
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            PickList::new(
                &mut self.page_format_sett_pl,
                Cow::from(&*ALL_FORMATS),
                Some(self.page_format_sett.to_string()),
                Message::SettingsPageFormat
            )
            .style(self.theme.clone())
        )
        .push(Space::with_height(Length::Units(settings_spacing)));

        col = col.push(Text::new("Save to:")) //Save Directory
        .push(Space::with_height(Length::Units(settings_mini_spacing)))
        .push(
            TextInput::new(
                &mut self.save_dir_sett_in,
                "Enter the Default Save Directory Here",
                &self.save_dir_sett,
                Message::SettingsSaveDir
            )
            .width(item_width)
            .style(self.theme.clone())
        ).push(Space::with_height(Length::Units(60)));

        col = col.push( //Save Button
            Button::new(
                &mut self.save_sett_button,
                Text::new("Save Settings").size(40)
            )
            .on_press(Message::SaveSettings)
            .style(self.theme.clone())
        );

        if self.err {
            col = col.push(Space::with_height(Length::Units(45)))
            .push(Text::new(&self.err_msg));
        }

        Container::new(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(self.theme.clone())
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
        set_err_msgs(Vec::new());
        self.err = false;
        self.err_msg = String::new();
    }

    fn new_from_prefs(prefs: Preferences) -> Self {
        let mut gui = Gui::default();
        
        if prefs.theme == &*ALL_THEMES[0] {
            gui.theme = Theme::Light;
        }
        else if prefs.theme == &*ALL_THEMES[1] {
            gui.theme = Theme::Dark;
        }

        for format in &*ALL_FORMATS {
            if prefs.format == *format {
                gui.page_format = &format;
                gui.page_format_sett = &format;
            }
        }

        if prefs.word_count >= 10 && prefs.word_count <= 20 {
            gui.word_count = prefs.word_count;
            gui.word_count_sett = prefs.word_count.to_string();
        }
        if prefs.letter_count >= 8 && prefs.letter_count <= 14 {
            gui.letter_count = prefs.letter_count;
            gui.letter_count_sett = prefs.letter_count.to_string();
        }
        gui.save_dir = prefs.save_directory.clone();
        gui.save_dir_sett = prefs.save_directory;

        gui
    }

    fn default() -> Self {
        Gui {
            letter_count_sl: slider::State::new(),
            letter_count: 11,
            word_count_sl: slider::State::new(),
            word_count: 15,
            page_format_pl: pick_list::State::default(),
            page_format: &ALL_FORMATS[0],
            save_dir_in: text_input::State::new(),
            save_dir: String::from(""),
            go_to_settings_button: button::State::new(),

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

            progress_state: ProgressState::Creating,
            finished_button: button::State::new(),

            err_msg: String::from(""),
            err: false,

            theme: Theme::Light,

            theme_sett_pl: pick_list::State::default(),
            theme_sett: &ALL_THEMES[0],
        	word_count_sett_in: text_input::State::new(),
            word_count_sett: String::from(""),
            letter_count_sett_in: text_input::State::new(),
            letter_count_sett: String::from(""),
            page_format_sett_pl: pick_list::State::default(),
            page_format_sett: &ALL_FORMATS[0],
            save_dir_sett_in: text_input::State::new(),
            save_dir_sett: String::from(""),
            save_sett_button: button::State::new(),
            return_button: button::State::new()
        }
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

fn set_err_msgs(msgs: Vec<String>) {
    match ERR_MSGS.lock() {
        Ok(mut val) => *val = msgs,
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            set_err_msgs(msgs);
        }
    }
}

fn get_err_msgs() -> Vec<String> {
    match ERR_MSGS.lock() {
        Ok(val) => val.clone(),
        Err(_) => {
            thread::sleep(std::time::Duration::from_millis(50));
            get_err_msgs()
        }
    }
}