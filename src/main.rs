use std::io;
use std::env;
use std::fmt;
use strum_macros::EnumString;
use std::str::FromStr;
use rusqlite::{params, Connection};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const USAGE: &str = "Usage: todo [options...]\n
\n
 -h, --help                                Show help options \n
 -i, --init                                Initializes database \n
 -r, --reset                               Resets database \n
 -a, --add 'title' 'description'           Adds new task \n
 -l, --list                                Lists all tasks \n
 -u, --update 'id' 'title' 'description'   Updates task\n
 -d, --delete 'id'                         Deletes task"
;

#[derive(Debug, PartialEq, EnumString)]
enum Status {
    TODO,
    DOING, 
    DONE
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::TODO => write!(f, "TODO"),
            Status::DOING => write!(f, "DOING"),
            Status::DONE => write!(f, "DONE"),
        }
    }
}

#[derive(Debug)]
struct Task {
    id: i64,
    title: String, 
    description: String,
    status: Status,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let connection = Connection::open("todo.db").expect("Connection to DB");
 
    let method: &str = &args[1];
    let arguments = &args[2..];
    match method {
        "-i" | "--init" => {
            let query = "create table if not exists tasks (title text not null, status text check(status in ('TODO', 'DOING', 'DONE')), description text);";
            match connection.execute(query, []) {
                Ok(_) => println!("Initialized database"),
                Err(_) => println!("Could not initialize database")
            };
        },
        "-r" | "--reset" => {
            println!("Are you sure you want to reset the database? (y|N)");
            let input = &mut String::new();
            io::stdin().read_line(input).expect("failed to read input");
            match input.trim() {
                "y" | "yes" => {
                    let query = "drop table tasks;";
                    match connection.execute(query, []) {
                        Ok(_) => println!("Reset database"),
                        Err(_) => println!("Could not reset database")
                    };
                },
                _ => println!("Process cancelled"),
            };
        },
        "-a" | "--add" => {
            if arguments.len() == 2 {
                match connection.execute("INSERT INTO tasks (title, status, description) values (?1, ?2, ?3);", params![&arguments[0], Status::TODO.to_string(), &arguments[1]]) {
                    Ok(result) => {
                        if result == 1 {
                            println!("Added task");
                        }else {
                            println!("Could not add task");
                        }
                    }, 
                    Err(_) => println!("Could not add task.\n-a, --add 'title' 'description'")
                };    
            }else {
                println!("Invalid arguments.\nFormat must be: -a, --add 'title' 'description'");
            }
        },
        "-l" | "--list" => {
            let mut stmt = connection.prepare("SELECT rowid, * FROM tasks;").expect("list tasks");
            
            match stmt.query_map([], |row| {
                Ok(Task {
                    id: row.get("rowid")?,
                    title: row.get("title")?,
                    description: row.get("description")?,
                    status: Status::from_str(&row.get::<&str, String>("status")?).expect("parse db row to Status enum")
                })
            }) {
                Ok(tasks) => println!("{:?}", tasks.collect::<Vec<_>>()), 
                Err(_) => println!("Could not list tasks. \n -l, --list")
            };
        },
        "-u" | "--update" => {
            if arguments.len() == 4 {
                match connection.execute("UPDATE tasks SET title = ?2, status = ?3, description = ?4 WHERE rowid == ?1;", params![&arguments[0], &arguments[1], &arguments[2], &arguments[3]]) {
                    Ok(result) => {
                        if result == 1 {
                            println!("Updated task with id = {}", &arguments[0]);
                        }else {
                            println!("Could not find or update task with id = {}", &arguments[0]);
                        }
                    }, 
                    Err(_) => println!("Could not update task.\n -u, --update 'id' 'title' 'description'")
                };  
            }else {
                println!("Invalid arguments.\nFormat must be: -u, --update 'id' 'title' 'description'");
            }
        },
        "-d" | "--delete" => {
            if arguments.len() == 1 {
                match connection.execute("DELETE FROM tasks WHERE rowid == ?1;", params![&arguments[0]]) {
                    Ok(result) => {
                        if result == 1 {
                            println!("Deleted task with id = {}", &arguments[0]);
                        }else {
                            println!("Could not find or delete task with id = {}", &arguments[0]);
                        }
                    }, 
                    Err(_) => println!("Could not delete task.\n -d, --delete 'id'")
                };
            }else{  
                println!("Invalid arguments.\nFormat must be: -d, --delete 'id'");
            }
        },
        "-v" | "--version" => {
            println!("ToDo {:?}", VERSION)
        },
        "-h" | "--help" | _ => {
            println!("{}", USAGE)
        },
    }
}
