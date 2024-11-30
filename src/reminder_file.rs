use crate::Reminder;
use chrono::Local;
use notify::{EventKind, RecursiveMode, Watcher};
use std::{fs::OpenOptions, io::Write, path::Path};

fn fuzzy_score(match_str: &str, search_str: &str) -> usize {
    let mut score = 0;
    if match_str.is_empty() || search_str.is_empty() {
        return score;
    }
    let search_str = search_str.to_lowercase();
    let match_str = match_str.to_lowercase();
    let mut match_iter = match_str.chars();
    let mut last_match_pos = 0;
    while let Some(mut match_char) = match_iter.next() {
        for (i, search_char) in search_str.chars().skip(last_match_pos).enumerate() {
            if match_char == search_char {
                score += 8;
                last_match_pos = i;
                if let Some(c) = match_iter.next() {
                    match_char = c;
                } else {
                    break;
                }
            } else if score > 0 {
                score -= 1;
            }
        }
    }
    score
}

pub struct ReminderFile {
    file: String,
    history_dir: String,
    reminders: Vec<Reminder>,
}

impl ReminderFile {
    const MAX_HISTORY: usize = 8;

    pub fn new<P: AsRef<Path>>(file: P, history_dir: P) -> Self {
        Self {
            file: file.as_ref().to_string_lossy().to_string(),
            history_dir: history_dir.as_ref().to_string_lossy().to_string(),
            reminders: Vec::new(),
        }
    }
    // appends directly to file
    pub fn append(&self, reminder: &Reminder) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)
            .unwrap();
        file.write_all(reminder.serialize().as_bytes())
            .expect("failed to add reminder");
        println!("added: {reminder}");
    }

    fn save_file(&self, file: &str) {
        let reminders_str = self
            .reminders
            .iter()
            .map(|rem| rem.serialize())
            .collect::<String>();
        std::fs::write(file, reminders_str).unwrap_or_default();
    }

    fn load_file(&self, file: &str) -> Vec<Reminder> {
        let reminder_str = std::fs::read_to_string(file).unwrap_or_default();
        let reminder_str = reminder_str.trim();
        let reminder_lines = reminder_str.lines();
        reminder_lines.map(Reminder::deserialize).collect()
    }

    pub fn save(&self) {
        self.save_file(&self.file);
    }

    pub fn save_history(&self) {
        let history_files = std::fs::read_dir(&self.history_dir)
            .unwrap()
            .map(|res| res.unwrap().path())
            .collect::<Vec<_>>();
        if history_files.len() >= Self::MAX_HISTORY {
            std::fs::remove_file(history_files.into_iter().min().unwrap()).unwrap();
        }
        let now = Local::now();
        self.save_file(&format!(
            "{}\\reminders {}.txt",
            self.history_dir,
            now.format("%y-%m-%d %H-%M-%S.%3f")
        ));
    }

    pub fn undo(&mut self) {
        let history_files = std::fs::read_dir(&self.history_dir)
            .unwrap()
            .map(|res| res.unwrap().path())
            .collect::<Vec<_>>();
        if let Some(last_file) = history_files.into_iter().max() {
            self.reminders = self.load_file(last_file.to_str().unwrap());
            self.save();
            std::fs::remove_file(&last_file).unwrap();
        }
    }

    pub fn load(&mut self) {
        self.reminders = self.load_file(&self.file);
    }

    fn match_title(&self, title: &str) -> Option<usize> {
        self.reminders
            .iter()
            .enumerate()
            .filter_map(|(i, rem)| {
                let score = fuzzy_score(title, &rem.title);
                (score > 0).then_some((i, score))
            })
            .max_by_key(|(_, score)| *score)
            .map(|(i, _)| i)
    }

    fn closest_reminder(&self) -> Option<usize> {
        self.reminders
            .iter()
            .enumerate()
            .min_by_key(|(_, rem)| rem.end_time)
            .map(|(i, _)| i)
    }

    pub fn remove_line(&mut self, line: usize) {
        self.reminders.remove(line);
        self.save();
    }

    pub fn remove(&mut self, title: &str) {
        if let Some(best_match_idx) = self.match_title(title) {
            println!("removed: {}", &self.reminders[best_match_idx]);
            self.remove_line(best_match_idx);
        } else {
            println!("no reminders with title \"{title}\" found");
        }
    }

    pub fn skip(&mut self, title: &str, skips: u32) {
        if let Some(best_match_idx) = self.match_title(title) {
            self.reminders[best_match_idx].skips += skips;
            self.save();
        } else {
            println!("no reminders with title \"{title}\" found");
        }
    }

    pub fn skip_next(&mut self, skips: u32) {
        if let Some(i) = self.closest_reminder() {
            self.reminders[i].skips += skips;
            self.save();
        } else {
            println!("no next reminder");
        }
    }

    pub fn wait_next(&mut self) {
        if !Path::new(&self.file).exists() {
            std::fs::File::create_new(&self.file).unwrap();
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();
        watcher
            .watch(Path::new(&self.file), RecursiveMode::NonRecursive)
            .unwrap();
        self.load();
        println!("reminders loaded from file: {}", self.reminders.len());
        loop {
            if let Ok(res) = rx.try_recv() {
                match res {
                    Ok(e) => match e.kind {
                        EventKind::Create(create_kind) => {
                            if create_kind == notify::event::CreateKind::File {
                                self.load();
                            }
                        }

                        EventKind::Modify(modify_kind) => match modify_kind {
                            notify::event::ModifyKind::Any | notify::event::ModifyKind::Data(_) => {
                                self.load();
                            }
                            notify::event::ModifyKind::Name(_) => {
                                self.reminders.clear();
                            }
                            _ => {}
                        },
                        EventKind::Remove(remove_kind) => {
                            if remove_kind == notify::event::RemoveKind::File {
                                self.reminders.clear();
                            }
                        }
                        _ => {}
                    },
                    Err(e) => eprintln!("{e}"),
                }
            }
            if let Some(i) = self.closest_reminder() {
                let closest_reminder = &mut self.reminders[i];
                let (updated, should_remove) = closest_reminder.update();
                if updated {
                    if closest_reminder.skips > 0 {
                        closest_reminder.skips -= 1;
                    } else {
                        notify_rust::Notification::new()
                            .summary(&closest_reminder.title)
                            .show()
                            .unwrap();
                    }
                    watcher.unwatch(Path::new(&self.file)).unwrap();
                    if should_remove {
                        self.remove_line(i);
                    } else {
                        self.save();
                    }
                    watcher
                        .watch(Path::new(&self.file), RecursiveMode::NonRecursive)
                        .unwrap();
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    pub fn list(&mut self) {
        for reminder in self.reminders.iter() {
            println!("{reminder}");
        }
    }

    pub fn clear(&mut self) {
        if self.reminders.is_empty() {
            return;
        }
        println!("cleared:");
        self.list();
        self.reminders.clear();
        self.save();
    }
}
