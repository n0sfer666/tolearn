use std::path::{Component, Path, PathBuf};

use crate::error::CliError;

pub fn apart(target: &Path, app: &Path) -> Result<(), CliError> {
    if !resolved(target).starts_with(resolved(app)) {
        return Ok(());
    }
    Err(CliError::Data(format!(
        "`{}` лежит в хранилище приложения `{}`: туда пишут только генерация и импорт в приложении, назови каталог вне его",
        target.display(),
        app.display()
    )))
}

fn resolved(path: &Path) -> PathBuf {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    for base in absolute.ancestors() {
        let (Ok(real), Ok(rest)) = (base.canonicalize(), absolute.strip_prefix(base)) else {
            continue;
        };
        return rest.components().fold(real, |mut path, part| {
            match part {
                Component::ParentDir => {
                    path.pop();
                }
                Component::Normal(name) => path.push(name),
                Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
            }
            path
        });
    }
    absolute
}
