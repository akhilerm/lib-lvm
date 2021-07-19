use std::error::Error;
use std::process::Command;

#[derive(Debug)]
pub struct PoolCreateReq {
    pub name: String,
    pub devices: Vec<String>,
}

#[derive(Debug)]
pub struct Pool {
    pub name: String,
    pub devices: Vec<String>,
    pub capacity: u64,
    pub used: u64,
}



pub(crate) fn create_vg(req: PoolCreateReq) -> Result<Pool, Box<dyn Error>> {
    let pool_name = req.name.as_str();
    let output = Command::new("pvcreate")
        .args(&req.devices)
        .output()
        .expect("failed execute pv_create");
    // TODO: check output and return custom error 
    let output = Command::new("vgcreate")
        .arg(pool_name)    
        .args(&req.devices)
        .output()
        .expect("failed execute vg_create");
     // TODO: check output and return custom error 
    let output = Command::new("vgs")
        .arg(pool_name)
        .args(&["--options=vg_size,vg_free", "--units=b", "--noheadings", "--nosuffix"])
        .output()
        .expect("failed execute vgs");
    let s = std::str::from_utf8(output.stdout.as_slice())?;
    let size = s.trim().split(" ").collect::<Vec<&str>>();
    let capacity = size[0].parse::<u64>()?;
    let free = size[1].parse::<u64>()?;
    let pool = Pool{
        name: pool_name.to_string(),
        devices: req.devices,
        capacity: capacity,
        used: capacity - free,
    };
    Ok(pool)
}

fn remove_vg() {
    todo!()
}

fn create_lvm_vol() {
    todo!()
}

fn remove_lvm_vol() {
    todo!()
}