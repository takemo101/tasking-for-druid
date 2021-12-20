use druid::im::{Vector};
use druid::{Data};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::env;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Data)]
pub enum TaskStatus {
    New,
    Progress,
    Stop,
    Done,
}

impl TaskStatus {
    pub fn to_string(&self) -> &str {
        match self {
            Self::New => "新規",
            Self::Progress => "実行中",
            Self::Stop => "停止",
            Self::Done => "完了",
        }
    }

    pub fn next_status(&self) -> Self {
        match self {
            Self::New => Self::Progress,
            Self::Progress => Self::Stop,
            Self::Stop => Self::Done,
            Self::Done => Self::New,
        }
    }
}

impl PartialEq for TaskStatus {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Data)]
pub struct Task {
    pub id: u32,
    pub content: String,
    pub status: TaskStatus,
}

impl Task {
    /// create task
    pub fn create(id: u32, content: String) -> Self {
        Self {
            id: id,
            content: content,
            status: TaskStatus::New,
        }
    }

    /// change task status
    pub fn change_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    /// task equals
    pub fn equals(&self, id: u32) -> bool {
        self.id == id
    }
}

/// save task collection
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SaveTasks {
    pub id_counter: u32,
    pub tasks: Vec<Task>,
}

impl SaveTasks {
    pub fn new() -> Self {
        Self {
            id_counter: 0,
            tasks: Vec::new(),
        }
    }

    /// convert vec to vector
    pub fn to_vector(&self) -> Vector<Task> {
        let mut vec: Vector<Task> = Vector::new();
        for task in self.tasks.iter() {
            vec.push_back(task.clone());
        }

        vec
    }
}

/// task collection
#[derive(Default, Debug, Clone, Data)]
pub struct Tasks {
    pub id_counter: u32,
    pub tasks: Vector<Task>,
}

impl Tasks {
    /// construct
    pub fn new() -> Self {
        Self {
            id_counter: 0,
            tasks: Vector::new(),
        }
    }

    pub fn from_save_tasks(tasks: SaveTasks) -> Self {
        Self {
            id_counter: tasks.id_counter,
            tasks: tasks.to_vector(),
        }
    }

    /// find task by id
    pub fn find_by_id(&mut self, id: u32) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.equals(id))
    }

    /// add task message
    pub fn add_message(&mut self, message: String) {
        let id = self.generate_task_id();
        self.tasks.push_back(Task::create(id , message));
    }

    /// remove task by id
    pub fn remove_by_id(&mut self, id: u32) {
        self.tasks.retain(|task| !task.equals(id));
    }

    /// clear all tasks
    pub fn clear(&mut self) {
        self.tasks.retain(|_| false);
    }

    /// is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// convert vector to vec
    pub fn to_vec(&self) -> Vec<Task> {
        let mut vec: Vec<Task> = Vec::new();
        for task in self.tasks.iter() {
            vec.push(task.clone());
        }

        vec
    }

    /// convert tasks to save_tasks
    pub fn to_save_tasks(&self) -> SaveTasks {
        return SaveTasks {
            id_counter: self.id_counter,
            tasks: self.to_vec(),
        }
    }

    /// generate task_id
    fn generate_task_id(&mut self) -> u32 {
        self.id_counter += 1;
        self.id_counter
    }
}

/// task json repository
#[derive(Default, Data, Clone)]
pub struct TaskRepository {
    filename: String,
}

impl TaskRepository {
    /// construct
    pub fn new(filename: String) -> Self {

        let path_buf = env::current_exe().expect("カレントパスを取得できませんでした");
        println!("{}", path_buf.display());
        if let Some(path) = path_buf.parent() {
            let pb = path.join(filename.clone());
            if let Some(p) = pb.to_str() {
                println!("{}", p.to_string());
                return Self {
                    filename: p.to_string(),
                }
            }
        }

        Self {
            filename: filename,
        }
    }

    /// save tasks
    pub fn save(&self, tasks: SaveTasks) {
        let path = Path::new(self.filename.as_str());
        let serialized = serde_json::to_string(&tasks).expect("シリアライズできませんでした");
        let mut file = fs::File::create(path).expect(format!("{} ファイルが開けませんでした", path.display()).as_str());
        writeln!(file, "{}", serialized).expect("ファイルに書き込めませんでした");
    }

    /// load tasks from json
    pub fn load(&self) -> SaveTasks {
        if let Ok(serialized) = fs::read_to_string(Path::new(self.filename.as_str())) {
            if let Ok(tasks) = serde_json::from_str::<SaveTasks>(&serialized) {
                return tasks
            }
        }

        SaveTasks::new()
    }
}
