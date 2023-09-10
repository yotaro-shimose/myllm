use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use markdown::mdast::Node;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    text: String,
    done: bool,
    deadline: Option<NaiveDateTime>,
    completed_at: Option<NaiveDateTime>,
}

impl Task {
    fn from_node(node: &Node) -> Option<Self> {
        let icon = "📅";
        let (text, done) = Self::extract_task_text(node)?;
        let ret = Self::parse_iconed_datetime(&text, icon).or(Self::parse_iconed_date(&text, icon));
        let (daedline, text) = if let Some((datetime, text)) = ret {
            (Some(datetime), text)
        } else {
            (None, text)
        };

        let icon = "✅";
        let ret = Self::parse_iconed_datetime(&text, icon).or(Self::parse_iconed_date(&text, icon));
        let (completed_at, text) = if let Some((datetime, text)) = ret {
            (Some(datetime), text)
        } else {
            (None, text)
        };
        Some(Self {
            text,
            done,
            deadline: daedline,
            completed_at,
        })
    }

    fn extract_task_text(node: &Node) -> Option<(String, bool)> {
        if let Node::ListItem(node) = node {
            let done = node.checked?;
            let paragraph = node.children.iter().find_map(|node| {
                if let Node::Paragraph(paragraph) = node {
                    Some(paragraph)
                } else {
                    None
                }
            })?;
            let text = paragraph
                .children
                .iter()
                .find_map(|node| {
                    if let Node::Text(text) = node {
                        Some(text)
                    } else {
                        None
                    }
                })?
                .value
                .to_owned();
            Some((text, done))
        } else {
            None
        }
    }

    fn parse_iconed_datetime(text: &str, icon: &str) -> Option<(NaiveDateTime, String)> {
        let pattern = icon.to_string() + r" *\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}";
        let re = regex::Regex::new(&pattern).unwrap();
        let deadline_part = re.find(text)?.as_str().trim();
        let remain = text.replace(deadline_part, "");
        let deadline = deadline_part.replace(icon, "");
        let (datetime, _) =
            NaiveDateTime::parse_and_remainder(deadline.trim(), "%Y-%m-%d %H:%M:%S").ok()?;
        Some((datetime, remain))
    }

    fn parse_iconed_date(text: &str, icon: &str) -> Option<(NaiveDateTime, String)> {
        let pattern = icon.to_string() + r" *\d{4}-\d{2}-\d{2}";
        let re = regex::Regex::new(&pattern).unwrap();
        let deadline_part = re.find(text)?.as_str().trim();
        let remain = text.replace(deadline_part, "");
        let deadline = deadline_part.replace(icon, "");
        let (date, _) = NaiveDate::parse_and_remainder(deadline.trim(), "%Y-%m-%d").ok()?;
        Some((date.and_time(NaiveTime::MIN), remain))
    }
}

fn main() -> Result<()> {
    let in_path = Path::new("data/Mehrabian.md");
    let md = fs::read_to_string(in_path)?;
    let tree = markdown::to_mdast(&md, &markdown::ParseOptions::gfm()).unwrap();
    let mut nodes = vec![tree];
    let mut tasks = Vec::new();
    while let Some(node) = nodes.pop() {
        if let Some(task) = Task::from_node(&node) {
            tasks.push(task);
        }
        if let Some(children) = node.children() {
            nodes.extend(children.to_owned())
        }
    }
    for task in &tasks {
        println!("{:?}", task);
    }
    let out_path = Path::new("hoge.json");
    let tasks_json = serde_json::to_string(&tasks)?;
    fs::write(out_path, tasks_json)?;
    Ok(())
}
