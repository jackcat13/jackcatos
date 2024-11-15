use alloc::{boxed::Box, string::{String, ToString}};

pub const PATH_MAX_SIZE: usize = 108;

#[derive(Debug, Clone)]
pub struct PathRoot{
    pub drive_number: u8,
    pub first: Box<PathPart>,
}

#[derive(Debug, Clone)]
pub struct PathPart {
    pub part: String,
    pub next: Option<Box<PathPart>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PathValidation {
    Valid(u8, String),
    Invalid(String),
}

fn valid_path_format(path: String) -> PathValidation {
    if path.len() > PATH_MAX_SIZE {
        return PathValidation::Invalid("Path > 108 character, too long.".to_string());
    } else if path.len() <= 3 {
        return PathValidation::Invalid("Path <= 3 character, too short.".to_string());
    }
    
    let drive_number = path.chars().nth(0).unwrap();
    if drive_number.is_digit(10) == false {
        return PathValidation::Invalid("Path does not start with a digit.".to_string());
    }
    
    let drive_suffix = path.get(1..3).unwrap();
    if drive_suffix != ":/" {
        return PathValidation::Invalid("Path does not contain the drive suffix :/ after first digit.".to_string());
    }
    
    let path = path.get(3..).unwrap().to_string();
    PathValidation::Valid(drive_number.to_digit(10).unwrap() as u8, path)
}

pub fn init_path(path: String) -> Result<PathRoot, String> {
    match valid_path_format(path) {
        PathValidation::Valid(drive_number, path) => {
            let part = if !path.contains("/") { path.clone() } else { path.get(0..path.find("/").unwrap()).unwrap().to_string() };
            Ok(PathRoot{
                drive_number,
                first: Box::new(PathPart{
                    part,
                    next: _resolve_next(path),
                })
            })
        },
        PathValidation::Invalid(error) => Err(error),
    }
}

fn _resolve_next(path: String) -> Option<Box<PathPart>> {
    if path.contains("/") {
        let path = path.get(path.find("/").unwrap()+1..).unwrap().to_string();
        if path.trim() == "" { return None; }
        let part = if !path.contains("/") { path.clone() } else { path.get(0..path.find("/").unwrap()).unwrap().to_string() };
        Some(Box::new(PathPart{
            part,
            next: _resolve_next(path),
        }))
    } else {
        None
    }
}