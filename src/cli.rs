use std::env::Args;
use std::path::PathBuf;

use crate::error;

/// gets the canonicalized version of the given path
pub fn get_full_path<P>(path: P) -> error::Result<PathBuf>
where
    P: Into<PathBuf>
{
    let path = path.into();

    let to_check = if !path.is_absolute() {
        let mut cwd = std::env::current_dir()?;
        cwd.push(path);
        cwd
    } else {
        path
    };

    let rtn = std::fs::canonicalize(to_check)?;

    Ok(rtn)
}

/// attempts to retrieve the next argument
/// 
/// if the argument is not present then it will return an error indicating the
/// argument is missing and provide the name of the argument
pub fn get_arg_value<N>(args: &mut Args, name: N) -> error::Result<String>
where
    N: AsRef<str>
{
    let Some(v) = args.next() else {
        let mut msg = String::from("missing ");
        msg.push_str(name.as_ref());
        msg.push_str(" argument value");

        return Err(error::Error::new(error::ErrorKind::MissingArgument)
            .with_message(msg))
    };

    Ok(v)
}

pub fn get_file_reader<P>(file_path: Option<P>) -> error::Result<std::io::BufReader<std::fs::File>>
where
    P: Into<PathBuf>
{
    let Some(path_ref) = file_path else {
        return Err(error::build::no_file_provided())
    };

    let full_path = get_full_path(path_ref)?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(full_path)?;

    Ok(std::io::BufReader::new(file))
}