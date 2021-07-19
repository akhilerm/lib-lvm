use lvm::{Pool, PoolCreateReq};

use snafu::{Snafu};
use clap::{App, Arg, ArgMatches};

pub mod lvm;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Missing value for {}", field))]
    MissingValue { field: String },
}

fn main() {

    let matches = App::new("lvm-lib")
        .version("0.1")
        .about("lvm commands to create pools and volumes")
        .subcommand(App::new("vgcreate")
            .about("Create volume group")
            .arg(
                Arg::with_name("name")
                    .required(true)
                    .index(1)
                    .help("Volume group name"),
            )
            .arg(
                Arg::with_name("disks")
                    .required(true)
                    .multiple(true)
                    .index(2)
                    .help("Disk device files"),
            )
        )
        .get_matches();

    let status = match matches.subcommand() {
        ("vgcreate", Some(args)) => {
            let pool = create(args);
            println!("{:#?}", pool)
        },
        _ => panic!("Command not found"),
        
    };

}

fn create(
    matches: &ArgMatches<'_>,
) -> Result<lvm::Pool, Box<dyn std::error::Error>> {
    let name = matches
        .value_of("name")
        .ok_or_else(|| Error::MissingValue {
            field: "name".to_string(),
        })?
        .to_owned();
    let disks = matches
        .values_of("disks")
        .ok_or_else(|| Error::MissingValue {
            field: "disks".to_string(),
        })?
        .map(|dev| dev.to_owned())
        .collect();
    let mut req = PoolCreateReq{
            name: name,
            devices: disks,
        };
    let pool = lvm::create_vg(req)?;
    Ok(pool)
}