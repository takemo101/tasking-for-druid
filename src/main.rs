#![windows_subsystem = "windows"]

use druid::im::{Vector};
use druid::kurbo::{Insets};
use druid::widget::prelude::*;
use druid::widget::{Flex, Label, TextBox, List, Scroll, ViewSwitcher, Painter, FlexParams, CrossAxisAlignment};
use druid::{commands, theme, lens, AppDelegate, AppLauncher, Command, Handled, Target, DelegateCtx, Data, Lens, Color, Widget, LensExt, WidgetExt, WindowDesc};

mod model;
use model::{TaskStatus, Task, Tasks, TaskRepository};

const TEXT_SIZE: f64 = 18.0;
const BLOCK_HEIGHT: f64 = 38.0;
const BORDER_RADIUS: f64 = 4.0;
const WINDOW_HEIGHT: f64 = 500.0;
const WINDOW_WIDTH: f64 = 400.0;
const BLOCK_SPACE: f64 = 10.0;
const INNER_WIDTH: f64 = WINDOW_WIDTH - (BLOCK_SPACE * 2.0);
const LINE_HEIGHT: f64 = BLOCK_HEIGHT + (BLOCK_SPACE * 2.0);

const TASK_TEXT_SIZE: f64 = 14.0;
const TASK_BLOCK_HEIGHT: f64 = 24.0;

// save tasks json file
const SAVE_FILENAME: &str = "task.json";

#[derive(Clone, Data, Lens)]
struct TaskState {
    content: String,
    memo: String,
    tasks: Tasks,
    repository: TaskRepository,
    setting: bool,
}

fn main() {
    // describe the main window
    let main_window = WindowDesc::new(make_widget())
        .title("Tasking!")
        .resizable(false)
        .window_size((WINDOW_WIDTH, WINDOW_HEIGHT));

    let repository = TaskRepository::new(SAVE_FILENAME.to_string());

    // create the initial app state
    let initial_state: TaskState = TaskState {
        content: "".into(),
        memo: "".into(),
        tasks: Tasks::from_save_tasks(repository.load()),
        repository: repository,
        setting: false,
    };

    // start the application. Here we pass in the application state.
    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .log_to_console()
        .configure_env(|env, _| {
            env.set(theme::WINDOW_BACKGROUND_COLOR, Color::WHITE);
            env.set(theme::CURSOR_COLOR, Color::BLACK);
            env.set(theme::BACKGROUND_LIGHT, Color::rgba8(240, 240, 240, 0));
            env.set(theme::TEXT_COLOR, Color::BLACK);
            env.set(theme::TEXTBOX_BORDER_RADIUS, BORDER_RADIUS);
            env.set(theme::TEXTBOX_INSETS, Insets::new(8.5, 8.5, 8.5, 8.5));
        })
        .launch(initial_state)
        .expect("アプリケーションを起動できませんでした");
}

struct Delegate;

impl AppDelegate<TaskState> for Delegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        _: Target,
        cmd: &Command,
        _: &mut TaskState,
        _: &Env,
    ) -> Handled {
        if cmd.is(commands::CLOSE_WINDOW) {
            ctx.submit_command(commands::QUIT_APP);
            return Handled::Yes;
        }
        Handled::No
    }
}

fn make_widget() -> impl Widget<TaskState> {

    ViewSwitcher::new(
        |data: &TaskState, _| data.setting,
        |flag, _, _| {
            match flag {
                true => {
                    let mut panel = Flex::column();

                    panel.add_child(
                        TextBox::multiline()
                            .with_text_size(TASK_TEXT_SIZE)
                            .expand_width()
                            .fix_height(WINDOW_HEIGHT - LINE_HEIGHT - BLOCK_SPACE)
                            .lens(TaskState::memo)
                    );
                    panel.add_spacer(BLOCK_SPACE);
                    panel.add_child(
                        make_button::<TaskState>("戻る".to_string(), TEXT_SIZE,(88, 97, 105))
                            .expand_width()
                            .fix_height(BLOCK_HEIGHT)
                            .on_click(|_, data, _| {
                                data.setting = !data.setting;
                            }),
                    );

                    Box::new(
                        panel
                            .padding(BLOCK_SPACE),
                    )
                },
                _ => {
                    let mut column = Flex::column();
                    column.add_child(
                        Flex::row()
                            .with_flex_child(
                                TextBox::new()
                                    .with_placeholder("新しいタスクを入力してください")
                                    .with_text_size(TEXT_SIZE)
                                    .expand_width()
                                    .fix_height(BLOCK_HEIGHT)
                                    .lens(TaskState::content),
                                4.0,
                            )
                            .with_spacer(BLOCK_SPACE)
                            .with_flex_child(
                                make_button::<TaskState>("追加".to_string(), TEXT_SIZE,(0, 123, 255))
                                    .expand_width()
                                    .fix_height(BLOCK_HEIGHT)
                                    .on_click(|_, data, _| {
                                        if data.content.len() > 0 {
                                            data.tasks.add_message(data.content.to_string());
                                            data.content = "".to_string();
                                            data.repository.save(data.tasks.to_save_tasks());
                                        }
                                    }),
                                1.0,
                            )
                    );

                    column.add_spacer(BLOCK_SPACE);
                    column.add_child(
                        Scroll::new(
                            ViewSwitcher::new(
                                |data: &TaskState, _| data.tasks.clone(),
                                |tasks, _, _| {
                                    match tasks.is_empty() {
                                        true => {
                                            Box::new(
                                                Label::new("タスクはまだありません")
                                                    .with_text_size(TEXT_SIZE)
                                                    .center()
                                                    .fix_width(INNER_WIDTH)
                                                    .fix_height(WINDOW_HEIGHT - (LINE_HEIGHT * 2.0))
                                            )
                                        },
                                        _ => {
                                            Box::new(
                                                List::new(|| {
                                                    Flex::row()
                                                        .with_flex_child(
                                                            Label::new(|(_, item): &(TaskState, Task), _: &Env| item.content.to_string())
                                                                .with_text_size(TASK_TEXT_SIZE)
                                                                .padding(5.0)
                                                                .expand_width(),
                                                            5.0,
                                                        )
                                                        .with_spacer(BLOCK_SPACE)
                                                        .with_flex_child(
                                                            make_status_button(|(_, task): &(TaskState, Task), _| task.status.to_string().to_string(), TASK_TEXT_SIZE)
                                                                .on_click(|_, (tasks, task): &mut (TaskState, Task), _: &Env| {
                                                                    if let Some(t) = tasks.tasks.find_by_id(task.id) {
                                                                        t.change_status(task.status.next_status());
                                                                        tasks.repository.save(tasks.tasks.to_save_tasks());
                                                                    }
                                                                }),
                                                            1.2,
                                                        )
                                                        .with_spacer(BLOCK_SPACE)
                                                        .with_flex_child(
                                                            make_button("削除".to_string(), TASK_TEXT_SIZE, (255, 193, 7))
                                                                .on_click(|_, (tasks, task): &mut (TaskState, Task), _: &Env| {
                                                                    tasks.tasks.remove_by_id(task.id);
                                                                    tasks.repository.save(tasks.tasks.to_save_tasks());
                                                                }),
                                                            FlexParams::new(0.8, CrossAxisAlignment::End),
                                                        )
                                                        .with_spacer(BLOCK_SPACE)
                                                        .fix_height(TASK_BLOCK_HEIGHT)
                                                        .fix_width(INNER_WIDTH)
                                                })
                                                .with_spacing(BLOCK_SPACE)
                                                .lens(lens::Identity.map(
                                                    |d: &TaskState| (d.clone(), d.tasks.tasks.clone()),
                                                    |d: &mut TaskState, (state, _): (TaskState, Vector<Task>)| {
                                                        d.tasks = state.tasks;
                                                        d.content = state.content
                                                    },
                                                ))
                                            )
                                        },
                                    }
                                }
                            )
                        )
                            .fix_width(INNER_WIDTH)
                            .fix_height(WINDOW_HEIGHT - (LINE_HEIGHT * 2.0))
                    );

                    column.add_spacer(10.0);
                    column.add_child(
                        Flex::row()
                            .with_flex_child(
                                make_button::<TaskState>("クリア".to_string(), TEXT_SIZE,(108, 117, 125))
                                    .expand_width()
                                    .fix_height(38.0)
                                    .on_click(|_, data, _| {
                                        data.tasks.clear();
                                        data.repository.save(data.tasks.to_save_tasks());
                                    }),
                                    5.0,
                            )
                            .with_spacer(BLOCK_SPACE)
                            .with_flex_child(
                                make_button::<TaskState>("整頓".to_string(), TEXT_SIZE,(88, 97, 105))
                                    .expand_width()
                                    .fix_height(38.0)
                                    .on_click(|_, data, _| {
                                        data.tasks.sort(&vec![
                                            TaskStatus::New,
                                            TaskStatus::Progress,
                                            TaskStatus::Stop,
                                            TaskStatus::Done,
                                        ]);
                                        data.repository.save(data.tasks.to_save_tasks());
                                    }),
                                    1.0,
                            )
                            .with_spacer(BLOCK_SPACE)
                            .with_flex_child(
                                make_button::<TaskState>("メモ".to_string(), TEXT_SIZE,(88, 97, 105))
                                    .expand_width()
                                    .fix_height(38.0)
                                    .on_click(|_, data, _| {
                                        let tasks = data.tasks.to_vec();
                                        let mut text = "".to_string();

                                        for status in vec![
                                            TaskStatus::New,
                                            TaskStatus::Progress,
                                            TaskStatus::Stop,
                                            TaskStatus::Done,
                                        ].iter() {
                                            let mut task_text = format!("# {}タスク\n", status.to_string());
                                            let mut counter = 0;
                                            for task in tasks.iter() {
                                                if task.status.eq(status) {
                                                    counter += 1;
                                                    task_text = task_text + format!("{}. {}\n", counter, task.content).as_str();
                                                }
                                            }

                                            if counter != 0 {
                                                text = text + task_text.as_str() + "\n";
                                            }
                                        }

                                        text = text.trim().to_string();

                                        if text.len() > 0 {
                                            data.memo = format!("--- task ---\n{}\n------------", text).to_string();
                                        }

                                        data.setting = !data.setting;
                                    }),
                                    1.0,
                            )
                    );


                    Box::new(
                        column
                            .padding(BLOCK_SPACE),
                    )
                },
            }
        },
    )
}

fn make_button<T: Data>(label: String, text_size: f64, rgb: (u8, u8, u8)) -> impl Widget<T> {
    let painter = Painter::new(move |ctx, _, _| {
        let bounds = ctx.size().to_rounded_rect(BORDER_RADIUS);

        let (r, g, b) = rgb;

        if ctx.is_active() {
            ctx.fill(bounds, &Color::rgba8(r, g, b, 180));
        } else if ctx.is_hot() {
            ctx.fill(bounds, &Color::rgba8(r, g, b, 210));
        } else {
            ctx.fill(bounds, &Color::rgba8(r, g, b, 255));
        }
    });

    Label::new(label)
        .with_text_size(text_size)
        .with_text_color(Color::WHITE)
        .center()
        .background(painter)
}

fn make_status_button(label: fn(&(TaskState, Task), &Env) -> String, text_size: f64) -> impl Widget<(TaskState, Task)> {
    let painter = Painter::new(move |ctx, (_, task): &(TaskState, Task), _| {
        let bounds = ctx.size().to_rounded_rect(BORDER_RADIUS);

        let (r, g, b) = match task.status {
            TaskStatus::New => (23, 162, 184),
            TaskStatus::Progress => (40, 167, 69),
            TaskStatus::Stop => (108, 117, 125),
            TaskStatus::Done => (255, 0, 0),
        };

        if ctx.is_active() {
            ctx.fill(bounds, &Color::rgba8(r, g, b, 180));
        } else if ctx.is_hot() {
            ctx.fill(bounds, &Color::rgba8(r, g, b, 210));
        } else {
            ctx.fill(bounds, &Color::rgba8(r, g, b, 255));
        }
    });

    Label::new(label)
        .with_text_size(text_size)
        .with_text_color(Color::WHITE)
        .center()
        .background(painter)
}
