use fs_extra::dir::CopyOptions;
use mctk_core::component::{self, Component, RootComponent};
use mctk_core::event;
use mctk_core::layout::{Alignment, Dimension, Direction, Size};
use mctk_core::node;
use mctk_core::style::FontWeight;
use mctk_core::style::Styled;
use mctk_core::widgets::{Button, Div, IconButton, IconType, Image, Text, TextBox};
use mctk_core::widgets::{HDivider, Scrollable};
use mctk_core::{lay, msg, rect, size, size_pct, txt, Color};
use mctk_macros::{component, state_component_impl};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct FileManagerParams {}

#[derive(Debug, Clone)]
pub enum Message {
    GoBack,
    SelectEntry(PathBuf),
    DeleteSelected,
    CreateFolder,
    RenameSelected,
    CopySelected,
    Paste,
    OpenModal(bool),
    OpenFolerModal(bool),
    OpenActionModal(bool),
    OpenDeleteModal(bool),
    ConfirmAction,
    ConfirmDelete,
    UpdateFolderName(String),

}

#[derive(Debug)]
pub struct FileManagerState {
    current_path: PathBuf,
    entries: Vec<PathBuf>,
    selected_file: Option<PathBuf>,
    copied_file: Option<PathBuf>,
    message: String,
    file_viewer_open: bool,
    view_file: Option<PathBuf>,
    file_content: Option<String>,
    file_is_image: bool,
    file_is_pdf: bool,
    file_no_preview: bool,
    is_modal_open: bool,
    is_folder_options_modal:bool,
    is_action_modal_open: bool, // New field for the action modal
    action_modal_title: String,
    is_delete_modal_open: bool, // New field for the delete modal
    delete_item_name: String,
    folder_name: String,
    disable_click: bool,
}

#[component(State = "FileManagerState")]
#[derive(Debug, Default)]
pub struct FileManager {}

pub fn read_entries(path: PathBuf) -> Vec<PathBuf> {
    let mut entries = Vec::new();

    if let Ok(dir) = fs::read_dir(&path) {
        for entry in dir.flatten() {
            entries.push(entry.path());
        }
    } else {
        eprintln!("Failed to read directory: {:?}", path);
    }

    entries.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();

        if a_is_dir && !b_is_dir {
            std::cmp::Ordering::Less
        } else if !a_is_dir && b_is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.file_name().cmp(&b.file_name())
        }
    });

    entries
}

#[state_component_impl(FileManagerState)]
impl Component for FileManager {
    fn init(&mut self) {
        let current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let entries = read_entries(current_path.clone());

        self.state = Some(FileManagerState {
            current_path,
            entries,
            selected_file: None,
            copied_file: None,
            message: String::new(),
            file_viewer_open: false,
            view_file: None,
            file_content: None,
            file_is_image: false,
            file_is_pdf: false,
            file_no_preview: false,
            is_modal_open: false,
            is_folder_options_modal:false,
            is_action_modal_open: false, // Initialize action modal visibility
            action_modal_title: "".to_string(),
            is_delete_modal_open: false, // Initialize delete modal visibility
            delete_item_name: "".to_string(),
            folder_name: "".to_string(),
            disable_click: false,
        });

        self.state_ref();
    }

    fn update(&mut self, msg: component::Message) -> Vec<component::Message> {
        if let Some(m) = msg.downcast_ref::<Message>() {
            match m {
                Message::GoBack => {
                    if self.state_ref().file_viewer_open {
                        self.state_mut().file_viewer_open = false;
                        self.state_mut().view_file = None;
                        self.state_mut().file_content = None;
                        self.state_mut().file_is_image = false;
                        self.state_mut().file_is_pdf = false;
                        self.state_mut().file_no_preview = false;
                    } else {
                        if let Some(parent) = self.state_ref().current_path.parent() {
                            self.state_mut().current_path = parent.to_path_buf();
                            self.state_mut().message = "Went back.".to_string();
                            self.state_mut().entries =
                                read_entries(self.state_ref().current_path.clone());
                        } else {
                            self.state_mut().message = "No parent directory.".to_string();
                        }
                    }
                    self.state_ref();
                }

                Message::SelectEntry(path) => {
                    if path.is_dir() {
                        self.state_mut().selected_file = Some(path.clone());
                        self.state_mut().current_path = path.clone();
                        self.state_mut().message = "Entered directory.".to_string();
                        self.state_mut().entries =
                            read_entries(self.state_ref().current_path.clone());
                    } else {
                        self.state_mut().selected_file = Some(path.clone());
                        self.state_mut().file_viewer_open = true;
                        self.state_mut().view_file = Some(path.clone());
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        self.state_mut().file_is_image =
                            matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif");
                        self.state_mut().file_is_pdf = ext == "pdf";
                        self.state_mut().file_no_preview = false;
                        self.state_mut().file_content = None;

                        if self.state_mut().file_is_image {
                            // Handle image loading if necessary
                        } else if self.state_mut().file_is_pdf {
                            // Handle PDF loading if necessary
                        } else if ext == "txt" {
                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    self.state_mut().file_content = Some(content);
                                }
                                Err(_) => {
                                    self.state_mut().file_no_preview = true;
                                }
                            }
                        } else {
                            if let Ok(content) = fs::read_to_string(&path) {
                                self.state_mut().file_content = Some(content);
                            } else {
                                self.state_mut().file_no_preview = true;
                            }
                        }
                    }
                    self.state_ref();
                }

                Message::DeleteSelected => {
                    if let Some(selected) = &self.state_ref().selected_file {
                        self.state_mut().delete_item_name = selected
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        self.state_mut().is_delete_modal_open = true; // Open delete modal
                    } else {
                        self.state_mut().message = "No file/folder selected.".to_string();
                    }
                    self.state_ref();
                }

                Message::CreateFolder => {
                    self.state_mut().action_modal_title = "Create Folder".to_string();
                    self.state_mut().is_action_modal_open = true;
                    self.state_ref();
                }

                Message::RenameSelected => {
                    if let Some(_selected) = &self.state_ref().selected_file {
                        self.state_mut().action_modal_title = "Rename".to_string();
                        self.state_mut().is_action_modal_open = true;
                    }
                    self.state_ref();
                }

                Message::UpdateFolderName(name) => {
                    self.state_mut().folder_name = name.clone(); // Update folder name from TextBox
                    self.state_ref();
                }

                Message::CopySelected => {
                    if let Some(selected) = &self.state_ref().selected_file {
                        self.state_mut().copied_file = Some(selected.clone());
                        self.state_mut().message = "Copied to clipboard.".to_string();
                    } else {
                        self.state_mut().message = "No file/folder selected.".to_string();
                    }
                    self.state_ref();
                }

                Message::Paste => {
                    let state = self.state_mut();
                    if let Some(copied) = &state.copied_file {
                        let dest = state.current_path.join(copied.file_name().unwrap());

                        let res: io::Result<()> = if copied.is_dir() {
                            let opts = CopyOptions::new();
                            fs_extra::dir::copy(copied, &dest, &opts)
                                .map(|_| ())
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
                        } else {
                            fs::copy(copied, &dest)
                                .map(|_| ())
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
                        };

                        match res {
                            Ok(_) => {
                                state.message = "Pasted successfully.".to_string();
                            }
                            Err(e) => {
                                state.message = format!("Error pasting: {}", e);
                            }
                        }
                    } else {
                        state.message = "No file/folder copied.".to_string();
                    }

                    self.state_mut().entries = read_entries(self.state_ref().current_path.clone());
                    self.state_ref();
                }

                Message::OpenModal(value) => {
                    self.state_mut().is_modal_open = *value;
                    if *value {
                        self.state_mut().disable_click = true; // Disable clicks when modal is open
                    } else {
                        self.state_mut().disable_click = false; // Enable clicks when modal is closed
                    }
                    self.state_ref();
                }

                Message::OpenFolerModal(value) => {
                    self.state_mut().is_folder_options_modal = *value;
                    self.state_ref();
                }

                Message::OpenActionModal(value) => {
                    self.state_mut().is_action_modal_open = *value; // Open or close the action modal
                    self.state_mut().is_modal_open = false;
                    self.state_ref();
                    
                }

                Message::OpenDeleteModal(value) => {
                    self.state_mut().is_delete_modal_open = *value; // Open or close the action modal
                    self.state_mut().is_modal_open = false;
                    self.state_ref();
                }

                Message::ConfirmAction => {
                    match self.state_ref().action_modal_title.as_str() {
                        "Create Folder" => {
                            let folder_name = self.state_ref().folder_name.trim().to_string(); // Get the folder name and trim whitespace
                            if folder_name.is_empty() {
                                self.state_mut().message = "Folder name cannot be empty.".to_string();
                            } else {
                                let new_folder_path = self.state_ref().current_path.join(&folder_name);
                                if let Err(e) = fs::create_dir(&new_folder_path) {
                                    self.state_mut().message = format!("Error creating folder: {}", e);
                                } else {
                                    self.state_mut().message = format!("Created folder: {:?}", new_folder_path);
                                    self.state_mut().entries = read_entries(self.state_ref().current_path.clone());
                                }
                            }
                        }
                        "Rename" => {
                            let new_name = self.state_ref().folder_name.trim().to_string(); // Get the new name and trim whitespace
                            if new_name.is_empty() {
                                self.state_mut().message = "New name cannot be empty.".to_string();
                            } else if let Some(selected) = self.state_ref().selected_file.clone() {
                                let new_path = selected.with_file_name(new_name.clone());
                                if new_path.exists() {
                                    self.state_mut().message = format!("A file or folder named '{}' already exists.", new_name);
                                } else {
                                    if let Err(e) = fs::rename(&selected, &new_path) {
                                        self.state_mut().message = format!("Error renaming file: {}", e);
                                    } else {
                                        self.state_mut().message = format!("Renamed to: {:?}", new_path);
                                        self.state_mut().entries = read_entries(self.state_ref().current_path.clone());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    self.state_mut().is_action_modal_open = false; // Close modal after action
                    self.state_ref(); 
                }
                // Handle deletion confirmation
                Message::ConfirmDelete => {
                    if let Some(selected) = self.state_ref().selected_file.clone() {
                        let result = if selected.is_dir() {
                            fs::remove_dir_all(&selected)
                        } else {
                            fs::remove_file(&selected)
                        };

                        match result {
                            Ok(_) => {
                                self.state_mut().message =
                                    format!("Deleted: {:?}", self.state_ref().delete_item_name);
                                self.state_mut().selected_file = None;
                            }
                            Err(e) => {
                                self.state_mut().message = format!("Error deleting: {}", e);
                            }
                        }
                    }
                    self.state_mut().is_delete_modal_open = false; // Close delete modal
                    self.state_mut().entries = read_entries(self.state_ref().current_path.clone());
                    self.state_ref();
                }
            }
        }

        vec![]
    }

    fn view(&self) -> Option<mctk_core::Node> {
        let s = self.state_ref();

        if s.file_viewer_open {
            return Some(file_viewer_view(s));
        }

        let current_path = s.current_path.clone();
        let entries = s.entries.clone();

        let mut root = node!(
            Div::new().bg(Color::BLACK),
            lay![
                size_pct: [100],
                direction: Direction::Column,
                cross_alignment: Alignment::Stretch,
                padding: [5., 20., 5., 20.],
            ]
        );

        let current_folder_name = current_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let text_node = node!(Text::new(txt!(current_folder_name))
            .style("color", Color::rgb(197.0, 197.0, 197.0))
            .style("size", 28.0)
            .style("line_height", 20.)
            .style("font", "Space Grotesk")
            .style("font_weight", FontWeight::Normal));

            let header_node = node!(
                Div::new(),
                lay![
                    size_pct: [100, 10],
                    direction: Direction::Row,
                    axis_alignment: Alignment::Stretch,
                    cross_alignment: Alignment::Center,
                    margin: [5., 0., 0., 0.],
                ]
            )
            .push(
                node!(
                    Div::new(),
                    lay![
                        size_pct: [70, Auto],
                        axis_alignment: Alignment::Start,
                        cross_alignment: Alignment::Center,
                    ],
                )
                // .push(node!(
                //     Div::new(),
                //     lay![
                //             size_pct: [20, Auto],
                //             direction: Direction::Column,
                //             axis_alignment: Alignment::Start,
                //         ]
                // ))
                // .push(node!(
                //     Image::new(""),
                //     lay![
                //         size:[24,24],
                //         margin:[20.,0.,0.,10.]
                //     ]
                // ))
                .push(
                    node!(
                        Div::new(),
                        lay![
                            size_pct: [80, Auto],
                            direction: Direction::Column,
                            axis_alignment: Alignment::Start,
                        ]
                    )
                    .push(text_node),
                ),
            )
            .push(
                node!(
                    Div::new(),
                    lay![
                        size_pct: [30, Auto],
                        axis_alignment: Alignment::End,
                        padding: [0, 0, 0, 8.],
                    ]
                )
                .push(node!(
                    IconButton::new("add_icon")
                        .on_click(Box::new(|| msg!(Message::CreateFolder)))
                        .icon_type(IconType::Png)
                        .style(
                            "size",
                            Size {
                                width: Dimension::Px(30.0),
                                height: Dimension::Px(30.0),
                            }
                        )
                        .style("background_color", Color::TRANSPARENT)
                        .style("border_color", Color::TRANSPARENT)
                        .style("active_color", Color::rgba(85., 85., 85., 0.50))
                        .style("radius", 10.),
                    lay![
                        size: [42, 42],
                        axis_alignment: Alignment::End,
                        cross_alignment: Alignment::Center,
                    ]
                ))
                .push(node!(
                    IconButton::new("dots_icon") // Add the three-dots icon
                        .on_click(Box::new(|| msg!(Message::OpenFolerModal(true)))) // Open the options modal
                        .icon_type(IconType::Png)
                        .style(
                            "size",
                            Size {
                                width: Dimension::Px(30.0),
                                height: Dimension::Px(30.0),
                            }
                        )
                        .style("background_color", Color::TRANSPARENT)
                        .style("border_color", Color::TRANSPARENT)
                        .style("active_color", Color::rgba(85., 85., 85., 0.50))
                        .style("radius", 10.),
                    lay![
                        size: [42, 42],
                        axis_alignment: Alignment::End,
                        cross_alignment: Alignment::Center,
                    ]
                )),
            );
            let mut entries_div = node!(
                Div::new(),
                lay![
                    size: [440, Auto],
                    direction: Direction::Column,
                    cross_alignment: Alignment::Stretch,
                ]
            );
        
            // Create the modal for folder options
            let folder_options_modal = node!(
                Div::new().bg(Color::rgba(29., 29., 29., 1.)).border(
                    Color::rgba(127., 127., 135., 1.),
                    0.,
                    (10., 10., 10., 10.)
                ),
                lay![
                    size: [200, 250],
                    direction: Direction::Column,
                    position_type: Absolute,
                    position: [10., 210., 0., 0.],
                    cross_alignment: Alignment::Stretch,
                    axis_alignment: Alignment::Stretch,
                    padding: [10., 10., 10., 10.],
                ]
            )
            .push(
                node!(
                    Text::new(txt!("Folder Options"))
                        .style("color", Color::WHITE)
                        .style("size", 18.)
                        .style("line_height", 20.)
                        .style("font", "Space Grotesk")
                        .style("font_weight", FontWeight::Normal),
                )
            )
            .push(node!(HDivider {
                size: 1.,
                color: Color::MID_GREY
            }))
            .push(
                node!(
                    Button::new(txt!("Paste"))
                    .style("background_color", Color::TRANSPARENT)
                    .style("active_color", Color::MID_GREY)
                    .style("text_color", Color::WHITE)
                    .style("font_size", 16.0)
                    .style("line_height", 18.0)
                        .on_click(Box::new(|| msg!(Message::Paste))),
                    lay![margin: [0., 5., 5., 5.], size_pct: [100, 15]]
                )
            )
            .push(node!(HDivider {
                size: 0.3,
                color: Color::MID_GREY
            }))
            .push(
                node!(
                    Button::new(txt!("Delete"))
                    .style("background_color", Color::TRANSPARENT)
                    .style("active_color", Color::MID_GREY)
                    .style("text_color", Color::WHITE)
                    .style("font_size", 16.0)
                    .style("line_height", 18.0)
                        .on_click(Box::new(|| msg!(Message::DeleteSelected))),
                    lay![margin: [5., 5., 5., 5.], size_pct: [100, 15]]
                )
            )
            .push(node!(HDivider {
                size: 0.3,
                color: Color::MID_GREY
            }))
            .push(
                node!(
                    Button::new(txt!("Rename"))
                    .style("background_color", Color::TRANSPARENT)
                    .style("active_color", Color::MID_GREY)
                    .style("text_color", Color::WHITE)
                    .style("font_size", 16.0)
                    .style("line_height", 18.0)
                        .on_click(Box::new(|| msg!(Message::RenameSelected))),
                    lay![margin: [5., 5., 5., 5.], size_pct: [100, 15]]
                )
            )
            .push(node!(HDivider {
                size: 0.3,
                color: Color::MID_GREY
            }))
            .push(
                node!(
                    Button::new(txt!("Close"))
                    .style("background_color", Color::TRANSPARENT)
                    .style("active_color", Color::MID_GREY)
                    .style("text_color", Color::WHITE)
                    .style("font_size", 16.0)
                    .style("line_height", 18.0)
                        .on_click(Box::new(|| msg!(Message::OpenFolerModal(false)))),
                    lay![margin: [5., 5., 5., 5.], size_pct: [100, 15]]
                )

            );
            // .push(node!(HDivider {
            //     size: 1.,
            //     color: Color::MID_GREY
            // }));
            if s.is_folder_options_modal {
                entries_div = entries_div.push(folder_options_modal);
            }

        let back_row = Btnrow {
            title: "..".to_string(),
            value: "".to_string(),
            icon_1: "fold_icon".to_string(),
            icon_2: "".to_string(),
            color: Color::WHITE,
            on_click: Some(Box::new(move || Message::GoBack)),
            on_icon_2_click: Some(Box::new(move || {
                // This will open the action modal without selecting the entry
                msg!(Message::OpenActionModal(true));
                // Return a message indicating the action was triggered
                Message::OpenActionModal(true)
            })),
            is_modal_open: s.is_modal_open, // Pass the modal state
            is_folder_options_modal: s.is_folder_options_modal,
            is_action_modal_open: s.is_action_modal_open,
            is_delete_modal_open: s.is_delete_modal_open,
            disable_click: false, // Initialize the flag to allow clicks
        };

        entries_div = entries_div.push(node!(back_row));
        entries_div = entries_div.push(node!(HDivider {
            size: 0.5,
            color: Color::MID_GREY
        }));

        let action_modal = node!(
            Div::new().bg(Color::rgba(29., 29., 29., 1.)).border(
                Color::rgba(127., 127., 135., 1.),
                0.,
                (10., 10., 10., 10.)
            ),
            lay![
                size: [320, 160],
                direction: Direction::Column,
                position_type: Absolute,
                position: [140., 60., 0., 0.],
                cross_alignment: Alignment::Stretch,
                axis_alignment: Alignment::Stretch,
                padding: [15., 15.,  15., 10.]
            ]
        )
        .push(
            node!(
                Div::new(),
                lay![
                    size_pct: [100, 42],
                    cross_alignment: Alignment::Center,
                    axis_alignment: Alignment::Center,
                    direction: Direction::Column,
                ]
            )
            .push(node!(
                Text::new(txt!(s.action_modal_title.clone())) // Use the action modal title from state
                    .style("color", Color::WHITE)
                    .style("size", 18.)
                    .style("line_height", 20.)
                    .style("font", "Space Grotesk")
                    .style("font_weight", FontWeight::Normal),
                lay![
                    size: [Auto],
                ]
            )),
        )
        .push(
            // BUTTONS
            node!(
                Div::new(),
                lay![
                    size_pct: [100, 68],
                    direction: Direction::Row,
                    axis_alignment: Alignment::Stretch,
                    cross_alignment: Alignment::Stretch,
                ]
            )
            .push(node!(
                Div::new(),
                lay![
                    // size_pct: [28, 100],
                    axis_alignment: Alignment::Start,
                ]
            ))
            .push(
                node!(
                    Div::new(),
                    lay![
                        size_pct: [100, 100],
                        axis_alignment: Alignment::Center,
                        cross_alignment:Alignment::Center,
                        direction: Direction::Column,
                    ]
                )
                .push(node!(
                    TextBox::new(Some("".to_string()))
                        .with_class("text-md border-1 bg-transparent")
                        .placeholder("Enter Name")
                        .on_change(Box::new(|s| msg!(Message::UpdateFolderName(s.to_string())))),
                    lay![
                        size_pct: [80, 60],
                        margin: [0., 0., 10., 0.]
                    ]
                ))
                .push(
                    node!(
                        Div::new(),
                        lay![
                            size_pct: [100, 50],
                            axis_alignment: Alignment::Stretch,
                            direction: Direction::Row,
                            margin: [0., 0., 10., 0.]
                        ]
                    )
                    .push(node!(
                        Button::new(txt!("Cancel"))
                            .style("text_color", Color::WHITE)
                            .style("background_color", Color::rgba(68., 68., 68., 1.))
                            .style("active_color", Color::rgba(82., 81., 81., 1.))
                            .style("font_size", 16.)
                            .style("line_height", 18.)
                            .style("radius", 8.)
                            .on_click(Box::new(move || {
                                msg!(Message::OpenActionModal(false)) // Close the action modal
                            })),

                            // .on_click(Box::new(move || {
                            //     msg!(Message::OpenModal(false)) // Close modal
                            // })),
                        lay![
                            size_pct: [48, 100],
                            padding: [0., 0., 0., 12.],
                            cross_alignment: Alignment::Start,
                            axis_alignment: Alignment::Start,
                        ]
                    ))
                    .push(node!(
                        Button::new(txt!("Confirm"))
                            .style("text_color", Color::BLACK)
                            .style("background_color", Color::WHITE)
                            .style("active_color", Color::rgba(194., 184., 184., 1.))
                            .style("font_size", 16.)
                            .style("line_height", 18.)
                            .style("radius", 8.)
                            .on_click(Box::new(move || {
                                msg!(Message::ConfirmAction) // Trigger confirm action
                            })),
                        lay![
                            size_pct: [48, 100],
                            padding: [0., 12., 0., 0.],
                            axis_alignment: Alignment::End,
                        ]
                    )),
                ),
            ),
        );

        if s.is_action_modal_open {
            entries_div = entries_div.push(action_modal);
        }

        let delete_modal = node!(
            Div::new().bg(Color::rgba(29., 29., 29., 1.)).border(
                Color::rgba(127., 127., 135., 1.),
                0.,
                (10., 10., 10., 10.)
            ),
            lay![
                size: [320, 160],
                direction: Direction::Column,
                position_type: Absolute,
                position: [140., 60., 0., 0.],
                cross_alignment: Alignment::Stretch,
                axis_alignment: Alignment::Stretch,
                padding: [15., 15.,  15., 10.]
            ]
        )
        .push(
            node!(
                Div::new(),
                lay![
                    size_pct: [100, 72],
                    cross_alignment: Alignment::Center,
                    axis_alignment: Alignment::Center,
                ]
            )
            .push(node!(
                Text::new(txt!(format!("Delete {}?", s.delete_item_name)))
                    .style("color", Color::WHITE)
                    .style("size", 18.)
                    .style("line_height", 20.)
                    .style("font", "Space Grotesk")
                    .style("font_weight", FontWeight::Normal),
                lay![
                    size: [Auto],
                ]
            )),
        )
        .push(
            node!(
                Div::new(),
                lay![
                    size_pct: [100, 28],
                    direction: Direction::Row,
                    axis_alignment: Alignment::Stretch,
                    cross_alignment: Alignment::Stretch,
                ]
            )
            .push(node!(
                Div::new(),
                lay![
                    size_pct: [28, 100],
                    axis_alignment: Alignment::Start,
                ]
            ))
            .push(
                node!(
                    Div::new(),
                    lay![
                        size_pct: [72, 100],
                        axis_alignment: Alignment::Stretch,
                    ]
                )
                .push(node!(
                    Button::new(txt!("Cancel"))
                        .style("text_color", Color::WHITE)
                        .style("background_color", Color::rgba(68., 68., 68., 1.))
                        .style("active_color", Color::rgba(82., 81., 81., 1.))
                        .style("font_size", 16.)
                        .style("line_height", 18.)
                        .style("radius", 8.)
                        .on_click(Box::new(move || {
                            msg!(Message::OpenDeleteModal(false)) // Close modal
                        })),
                    lay![
                        size_pct: [48, 100],
                        padding: [0., 0., 0., 12.],
                        axis_alignment: Alignment::Start,
                    ]
                ))
                .push(node!(
                    Button::new(txt!("Confirm"))
                        .style("text_color", Color::BLACK)
                        .style("background_color", Color::WHITE)
                        .style("active_color", Color::rgba(194., 184., 184., 1.))
                        .style("font_size", 16.)
                        .style("line_height", 18.)
                        .style("radius", 8.)
                        .on_click(Box::new(move || {
                            msg!(Message::ConfirmDelete) // Trigger confirm action
                        })),
                    lay![
                        size_pct: [48, 100],
                        padding: [0., 12., 0., 0.],
                        axis_alignment: Alignment::End,
                    ]
                )),
            ),
        );

        if s.is_delete_modal_open {
            entries_div = entries_div.push(delete_modal);
        }

        if s.is_modal_open {
            let modal_content = node!(
                Div::new().bg(Color::BLACK), // border
                lay![
                    size: [200, 200],
                    direction: Direction::Column,
                    position_type: Absolute,
                    position:[120., 190.,0., 0],
                    cross_alignment: Alignment::Stretch,
                    axis_alignment:Alignment::Stretch,
                    // margin:[0., 10., 0., 0.],
                    padding: [20., 20., 20., 20.],
                ]
            )
            .push(node!(Text::new(txt!("File Options"))
                .style("color", Color::WHITE)
                .style("size", 20.0)
                .style("line_height", 24.0)
                .style("font", "Space Grotesk")))
            .push(node!(HDivider {
                size: 1.,
                color: Color::MID_GREY
            }))
            .push(node!(
                Button::new(txt!("Rename"))
                    .style("background_color", Color::TRANSPARENT)
                    .style("active_color", Color::MID_GREY)
                    .style("text_color", Color::WHITE)
                    .style("font_size", 16.0)
                    .style("line_height", 18.0)
                    .on_click(Box::new(|| msg!(Message::RenameSelected))),
                lay![margin: [5., 5., 5., 5.], size_pct: [100, 25]]
            ))
            .push(node!(HDivider {
                size: 0.3,
                color: Color::MID_GREY
            }))
            .push(node!(
                Button::new(txt!("Copy"))
                    .style("background_color", Color::TRANSPARENT)
                    .style("font_size", 15.0)
                    .style("line_height", 24.0)
                    .style("text_color", Color::WHITE)
                    .on_click(Box::new(|| msg!(Message::CopySelected))),
                lay![margin: [5., 5., 5., 5.], size_pct: [100, 25]]
            ))
            .push(node!(HDivider {
                size: 0.3,
                color: Color::MID_GREY
            }))
            .push(node!(
                Button::new(txt!("Delete"))
                    .style("background_color", Color::TRANSPARENT)
                    .style("font_size", 15.0)
                    .style("line_height", 24.0)
                    .style("text_color", Color::WHITE)
                    .on_click(Box::new(|| msg!(Message::DeleteSelected))),
                lay![margin: [5., 5., 5., 5.], size_pct: [100, 25]]
            ))
            .push(node!(HDivider {
                size: 0.3,
                color: Color::MID_GREY
            }))
            .push(node!(
                Button::new(txt!("Paste"))
                    .style("background_color", Color::TRANSPARENT)
                    .style("font_size", 15.0)
                    .style("line_height", 24.0)
                    .style("text_color", Color::WHITE)
                    .on_click(Box::new(|| msg!(Message::Paste))),
                lay![margin: [5., 5., 5., 5.], size_pct: [100, 25]]
            ))
            .push(node!(HDivider {
                size: 0.3,
                color: Color::MID_GREY
            }))
            .push(node!(
                Button::new(txt!("Close"))
                    .style("background_color", Color::TRANSPARENT)
                    .style("font_size", 15.0)
                    .style("line_height", 24.0)
                    .style("text_color", Color::WHITE)
                    .on_click(Box::new(|| msg!(Message::OpenModal(false)))),
                lay![margin: [5., 5., 5., 5.], size_pct: [100, 25]]
            ))
            .push(node!(HDivider {
                size: 1.,
                color: Color::MID_GREY
            }));

            entries_div = entries_div.push(modal_content);
        }

        for (i, entry) in entries.iter().enumerate() {
            let name = entry
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let entry_clone = Arc::new(entry.clone());
            let (main_icon, righticon) = if entry.is_dir() {
                ("fold_icon".to_string(), "".to_string()) // Replace with actual folder icon path
            } else {
                let ext = entry
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let file_icon = match ext.as_str() {
                    "txt" => "file_icon".to_string(),
                    "pdf" => "pdf_icon".to_string(),
                    "png" | "jpg" | "jpeg" | "gif" => "img_icon".to_string(),
                    _ => "file_icon".to_string(),
                };
                (file_icon, "dots_icon".to_string())
            };

            let btn_row = Btnrow {
                title: name.to_string(),
                value: "".to_string(),
                icon_1: main_icon,
                icon_2: righticon,
                color: Color::WHITE,
                on_click: Some(Box::new(move || {
                    Message::SelectEntry((*entry_clone).clone())
                })),
                on_icon_2_click: Some(Box::new(move || {
                    // This will open the action modal without selecting the entry
                    msg!(Message::OpenActionModal(true));
                    // Return a message indicating the action was triggered
                    Message::OpenActionModal(true)
                })),
                is_modal_open: s.is_modal_open, // Pass the modal state
                is_folder_options_modal: s.is_folder_options_modal,
                is_action_modal_open: s.is_action_modal_open,
                is_delete_modal_open: s.is_delete_modal_open,
                disable_click: false, // Initialize the flag to allow clicks
            };

            entries_div = entries_div.push(node!(btn_row).key(i as u64));
            entries_div = entries_div.push(
                node!(HDivider {
                    size: 0.5,
                    color: Color::MID_GREY
                })
                .key((i + 1) as u64),
            );
        }

        // let create_folder_btn = node!(
        //     Button::new(txt!("Create Folder"))
        //         .style("font_size", 15.0)
        //         .style("line_height", 24.0)
        //         .style("color", Color::WHITE)
        //         .on_click(Box::new(|| msg!(Message::CreateFolder))),
        //     lay![padding: [5., 5., 5., 5.], size_pct: [22, Auto]]
        // );

        // let rename_btn = node!(
        //     Button::new(txt!("Rename"))
        //         .style("font_size", 15.0)
        //         .style("line_height", 24.0)
        //         .style("color", Color::WHITE)
        //         .on_click(Box::new(|| msg!(Message::RenameSelected))),
        //     lay![padding: [5., 5., 5., 5.], size_pct: [15, Auto]]
        // );

        // let delete_btn = node!(
        //     Button::new(txt!("Delete"))
        //         .style("font_size", 15.0)
        //         .style("line_height", 24.0)
        //         .style("color", Color::WHITE)
        //         .on_click(Box::new(|| msg!(Message::DeleteSelected))),
        //     lay![margin: [5., 5., 5., 5.], size_pct: [15, Auto]]
        // );

        // let copy_btn = node!(
        //     Button::new(txt!("Copy"))
        //         .style("font_size", 15.0)
        //         .style("line_height", 24.0)
        //         .style("color", Color::WHITE)
        //         .on_click(Box::new(|| msg!(Message::CopySelected))),
        //     lay![margin: [5., 5., 5., 5.], size_pct: [10, Auto]]
        // );

        // let paste_btn = node!(
        //     Button::new(txt!("Paste"))
        //         .style("font_size", 15.0)
        //         .style("line_height", 24.0)
        //         .style("color", Color::WHITE)
        //         .on_click(Box::new(|| msg!(Message::Paste))),
        //     lay![margin: [5., 5.,  5., 5.], size_pct: [10, Auto]]
        // );

        // let back_btn = node!(
        //     Button::new(txt!("Go Back"))
        //         .style("font_size", 15.0)
        //         .style("line_height", 24.0)
        //         .style("color", Color::DARK_GREY)
        //         .on_click(Box::new(|| msg!(Message::GoBack))),
        //     lay![margin: [5., 0., 5., 5.], size_pct: [15, Auto]]
        // );

        // let mut actions_row = node!(
        //     Div::new().bg(Color::DARK_GREY),
        //     lay![
        //         direction: Direction::Row,
        //         position_type: Absolute,
        //         position: [Auto, 0., 0., 0.],
        //         size_pct: [100, 10],
        //         cross_alignment: Alignment::End,
        //         axis_alignment: Alignment::Center,
        //         padding: [0., 20., 0., 0.]
        //     ]
        // );

        // actions_row = actions_row
        //     .push(create_folder_btn)
        //     .push(rename_btn)
        //     .push(delete_btn)
        //     .push(copy_btn)
        //     .push(paste_btn)
        //     .push(back_btn);

        let mut scrollable_section = node!(
            Scrollable::new(size!(440, 380)),
            lay![
                size: [440, 380],
                direction: Direction::Column,
                cross_alignment: Alignment::Stretch,
            ]
        );
        scrollable_section = scrollable_section.push(entries_div);

        root = root.push(header_node);
        root = root.push(node!(HDivider {
            size: 1.,
            color: Color::MID_GREY
        }));
        root = root.push(scrollable_section);
        // root = root.push(actions_row);
        Some(root)
    }
}

impl RootComponent<FileManagerParams> for FileManager {}

pub struct Btnrow {
    pub title: String,
    pub value: String,
    pub icon_1: String,
    pub icon_2: String,
    pub color: Color,
    pub on_click: Option<Box<dyn Fn() -> Message + Send + Sync>>,
    pub on_icon_2_click: Option<Box<dyn Fn() -> Message + Send + Sync>>, // New field for icon 2 click
    pub is_modal_open: bool,
    pub is_action_modal_open: bool,
    pub is_delete_modal_open: bool,
    is_folder_options_modal:bool,
    pub disable_click: bool,
}

impl std::fmt::Debug for Btnrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Btnrow ")
            .field("title", &self.title)
            .field("icon_1", &self.icon_1)
            .field("icon_2", &self.icon_2)
            .finish()
    }
}
// self.disable_click 

impl Component for Btnrow {
    fn on_click(&mut self, event: &mut event::Event<event::Click>) {
        // Check if the modal is open
        if self.is_modal_open || self.is_action_modal_open || self.is_delete_modal_open || self.is_folder_options_modal || self.disable_click
        {
            return; // Ignore the click if the modal is open
        }
        

        if let Some(f) = &self.on_click {
            event.emit(Box::new(f()));
        }
    }

    fn view(&self) -> Option<node::Node> {
        let text_node = node!(Text::new(txt!(self.title.clone()))
            .style("color", self.color)
            .style("size", 20.0)
            .style("line_height", 20.)
            .style("font", "Space Grotesk")
            .style("font_weight", FontWeight::Normal));

        let value_node = node!(Text::new(txt!(self.value.clone()))
            .style("color", Color::rgb(197.0, 197.0, 197.0))
            .style("size", 20.0)
            .style("line_height", 20.)
            .style("font", "Space Grotesk")
            .style("font_weight", FontWeight::Normal));

        Some(
            node!(
                Div::new(),
                lay![
                    padding: [10, 10, 10, 10],
                    size_pct: [100, Auto],
                    direction: Direction::Row,
                    axis_alignment: Alignment::Stretch,
                    cross_alignment: Alignment::Center,
                ]
            )
            .push(
                node!(
                    Div::new(),
                    lay![
                        direction: Direction::Row,
                        cross_alignment: Alignment::Center,
                        axis_alignment: Alignment::Start,
                    ]
                )
                .push(node!(
                    Image::new(self.icon_1.clone()),
                    lay![
                        size:[24,24],
                        margin:[20.,0.,0.,10.]
                    ]
                ))
                .push(text_node),
            )
            .push(
                node!(
                    Div::new(),
                    lay![
                        direction: Direction::Row,
                        axis_alignment: Alignment::End,
                        cross_alignment:Alignment::Center,
                    ]
                )
                .push(value_node)
                .push(node!(
                    IconButton::new(self.icon_2.clone())
                        .on_click(Box::new(move || Box::new(Message::OpenModal(true)))) // Open modal when icon 2 is clicked
                        .icon_type(IconType::Png)
                        .style(
                            "size",
                            Size {
                                width: Dimension::Px(34.0),
                                height: Dimension::Px(34.0)
                            }
                        ),
                    lay![size:[34,34], margin:[10.,0.,0.,0.]]
                )),
            ),
        )
    }
}

// File viewer layout
fn file_viewer_view(s: &FileManagerState) -> node::Node {
    let file_name = s
        .view_file
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let header = node!(
        Div::new().bg(Color::BLACK),
        lay![
            direction: Direction::Row,
            cross_alignment: Alignment::Center,
            axis_alignment: Alignment::Start,
            padding: [5., 20., 5., 20.],
        ]
    )
    .push(node!(
        IconButton::new("back_icon")
            .on_click(Box::new(|| msg!(Message::GoBack)))
            .icon_type(IconType::Png)
            .style(
                "size",
                Size {
                    width: Dimension::Px(32.0),
                    height: Dimension::Px(34.0)
                }
            )
            .style("background_color", Color::TRANSPARENT)
            .style("border_color", Color::TRANSPARENT)
            .style("active_color", Color::rgba(85., 85., 85., 0.50)),
        lay![margin:[5.,5.,5.,5.], size:[32,34]]
    ))
    .push(node!(
        Text::new(txt!(file_name))
            .style("color", Color::WHITE)
            .style("size", 24.0)
            .style("line_height", 24.)
            .style("font", "Space Grotesk")
            .style("font_weight", FontWeight::Normal),
        lay![margin:[5.,20.,5.,5.]]
    ));

    let mut root = node!(
        Div::new().bg(Color::BLACK),
        lay![
            direction: Direction::Column,
            cross_alignment: Alignment::Stretch,
            axis_alignment: Alignment::Start,
            size_pct:[100,100]
        ]
    );

    root = root.push(header);
    root = root.push(node!(HDivider {
        size: 1.,
        color: Color::MID_GREY
    }));

    // Content area scrollable
    let mut content = node!(
        Div::new().bg(Color::BLACK),
        lay![
            direction: Direction::Column,
            cross_alignment: Alignment::Center,
            axis_alignment: Alignment::Start,
            padding:[20.,20.,20.,20. ],
            size_pct:[100,100],
        ]
    );

    if s.file_is_image {
        if let Some(file) = &s.view_file {
            let file_str = file.to_string_lossy().to_string();
            content = content.push(node!(
                Image::new(file_str),
                lay![size:[400,400], margin:[10.,10.,10.,10.]]
            ));
        } else {
            content = content.push(node!(Text::new(txt!("No file selected"))
                .style("color", Color::WHITE)
                .style("size", 18.0)
                .style("line_height", 24.0)
                .style("font", "Space Grotesk")));
        }
    } else if s.file_is_pdf {
        content = content.push(node!(Text::new(txt!("PDF viewing is not implemented"))
            .style("color", Color::WHITE)
            .style("size", 18.0)
            .style("line_height", 24.0)
            .style("font", "Space Grotesk")));
    } else if let Some(content_str) = &s.file_content {
        // Scrollable text area
        let mut scroll = node!(
            Scrollable::new(size!(440, 320)),
            lay![
                size: [440, 320],
                direction: Direction::Column,
                cross_alignment: Alignment::Stretch,
            ]
        );

        scroll = scroll.push(node!(
            Text::new(txt!(content_str.clone()))
                .style("color", Color::WHITE)
                .style("size", 14.0)
                .style("line_height", 20.0)
                .style("font", "Space Grotesk"),
            lay![margin:[5.,5.,5.,5.]]
        ));

        content = content.push(scroll);
    } else if s.file_no_preview {
        content = content.push(node!(Text::new(txt!(
            "No preview available for this file."
        ))
        .style("color", Color::WHITE)
        .style("size", 18.0)
        .style("line_height", 24.0)
        .style("font", "Space Grotesk")));
    } else {
        content = content.push(node!(Text::new(txt!("Loading or no file selected."))
            .style("color", Color::WHITE)
            .style("size", 18.0)
            .style("line_height", 24.0)
            .style("font", "Space Grotesk")));
    }

    root = root.push(content);
    root
}
