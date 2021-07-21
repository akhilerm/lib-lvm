use lvm::{PoolCreateReq};

extern crate serde;
#[macro_use]
extern crate serde_derive;

use snafu::{Snafu};
use clap::{App, Arg, ArgMatches};
use crate::lvm::{create_lvm_vol, CreateReplicaRequest};

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
        .subcommand(
      App::new("vgcreate")
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
        .subcommand(
            App::new("getvg")
            .about("Get a volume group")
            .arg(
                Arg::with_name("name")
                    .required(true)
                    .index(1)
                    .help("Volume group name"),
            )
        )
        .subcommand(
            App::new("removevg")
            .about("Delete a volume group")
            .arg(
                Arg::with_name("name")
                    .required(true)
                    .index(1)
                    .help("Volume group name"),
            )
        )
        .subcommand(
            App::new("listvg")
            .about("List volume group")
        )
        .subcommand(
            App::new("lvcreate")
            .about("create an LVM volume")
            .arg(
                Arg::with_name("vgname")
                    .required(true)
                    .index(1)
                    .help("volume group name")
            )
            .arg(
                Arg::with_name("lvname")
                    .required(true)
                    .index(2)
                    .help("name of the lvm volume")
            )
            .arg(
                Arg::with_name("size")
                    .required(true)
                    .index(3)
                    .help("size of the volume")
            )
        )
        .get_matches();

    let _status = match matches.subcommand() {
        ("vgcreate", Some(args)) => {
            let pool = vg_create(args);
            println!("{:#?}", pool)
        },
        ("getvg", Some(args)) => {
            let pool = get_vg(args);
            println!("{:#?}", pool)
        },
        ("removevg", Some(args)) => {
            let pool = remove_vg(args);
            println!("{:#?}", pool)
        },
        ("listvg", Some(args)) => {
            let pools = lvm::list_vg();
            println!("{:#?}", pools)
        },
        ("lvcreate", Some(args)) => {
            let replica = lv_create(args);
            println!("{:#?}", replica)
        }
        _ => panic!("Command not found"),
        
    };

}

fn vg_create(
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
    let req = PoolCreateReq{
            name: name,
            devices: disks,
        };
    let pool = lvm::create_vg(req)?;
    Ok(pool)
}

fn get_vg(
    matches: &ArgMatches<'_>,
) -> Result<lvm::Pool, Box<dyn std::error::Error>> {
    let name = matches
        .value_of("name")
        .ok_or_else(|| Error::MissingValue {
            field: "name".to_string(),
        })?
        .to_owned();
    let pool = lvm::get_vg(name)?;
    Ok(pool)
}

fn remove_vg(
    matches: &ArgMatches<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = matches
        .value_of("name")
        .ok_or_else(|| Error::MissingValue {
            field: "name".to_string(),
        })?
        .to_owned();
    lvm::remove_vg(name)?;
    Ok(())
}

fn lv_create(
    matches: &ArgMatches<'_>,
) -> Result<lvm::Replica, Box<dyn std::error::Error>> {
    let pool = matches
        .value_of("vgname")
        .ok_or_else(|| Error::MissingValue {
            field: "vgname".to_string(),
        })?
        .to_owned();
    let volume = matches
        .value_of("lvname")
        .ok_or_else(|| Error::MissingValue {
            field: "lvname".to_string(),
        })?
        .to_owned();
    let size = matches
        .value_of("size")
        .ok_or_else(|| Error::MissingValue {
            field: "size".to_string(),
        })?
        .to_owned()
        .parse::<u64>()
        .expect("failed to parse size");
    let mut req = CreateReplicaRequest{
        uuid: volume,
        pool,
        size,
        thin: false,
        share: 0
    };
    let volume = lvm::create_lvm_vol(req)?;
    Ok(volume)
}
