//! このモジュールは、コマンドライン引数の解析を定義します。
//! `clap`クレートを使用して、アプリケーションの様々なコマンドとサブコマンドを構造化します。

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "A Extended command-line tool, made from ipak.", long_about = None)]
#[command(name = "ipm")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

use crate::ipak::args::{
    PkgCommands, ProjectCommands, SystemCommands, UtilsCommands,
};

#[derive(Subcommand, Debug)]

pub enum Commands {
    /// Manage projects.
    #[command(subcommand)]
    Project(ProjectCommands),
    /// Configure system settings.
    #[command(subcommand)]
    System(SystemCommands),
    /// Utility commands.
    #[command(subcommand)]
    Utils(UtilsCommands),
    /// Manage packages.
    #[command(subcommand)]
    Pkg(PkgCommands),
    /// Manage Repositories
    #[command(subcommand)]
    Repo(RepoCommands),
}

use crate::utils::error::Error;
impl CommandExecution for Commands {
    fn exec(self) -> Result<(), Error> {
        use crate::ipak::args::CommandExecution as IpakCmdExec;
        match self {
            Self::Pkg(pkg_cmd) => {
                pkg_cmd.exec().map_err(Error::from)
            }
            Self::System(sys_cmd) => {
                sys_cmd.exec().map_err(Error::from)
            }
            Self::Project(proj_cmd) => {
                proj_cmd.exec().map_err(Error::from)
            }
            Self::Utils(utils_cmd) => {
                utils_cmd.exec().map_err(Error::from)
            }
            Self::Repo(repo_cmd) => repo_cmd.exec(),
        }
    }
}
pub trait CommandExecution {
    fn exec(self) -> Result<(), Error>;
}
#[derive(Subcommand, Debug)]
pub enum RepoCommands {}
impl CommandExecution for RepoCommands {
    fn exec(self) -> Result<(), Error> {
        crate::modules::repo::repo(self)
    }
}
