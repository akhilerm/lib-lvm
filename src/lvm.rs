use std::{process::Command};
use snafu::{Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to parse {}", err))]
    FailedParsing { err: String },
    #[snafu(display("Failed to execute command {}", err))]
    FailedExec { err: String },
}

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

pub(crate) fn create_vg(req: PoolCreateReq) -> Result<Pool, Error> {

    let output = Command::new("pvcreate")
        .args(&req.devices)
        .output()
        .expect("failed to execute pv_create");
    if !output.status.success() {
        let msg = match std::str::from_utf8(output.stderr.as_slice()){
            Ok(s) => s,
            Err(_) => "failed to execute pv_create",
        };
        return Err(Error::FailedExec{err: msg.to_string()})
    }
    
    let output = Command::new("vgcreate")
        .arg(req.name.as_str())    
        .args(&req.devices)
        .output()
        .expect("failed to execute vg_create");
    if !output.status.success() {
        let msg = match std::str::from_utf8(output.stderr.as_slice()){
            Ok(s) => s,
            Err(_) => "failed to execute vg_create",
        };
        return Err(Error::FailedExec{err: msg.to_string()})
    }
    
    get_vg(req.name)
}

#[derive(Debug, Serialize, Deserialize)]
struct VGListReport {
    report: Vec<VGList>
}
#[derive(Debug, Serialize, Deserialize)]
struct VGList {
    vg: Vec<VGName>
}
#[derive(Debug, Serialize, Deserialize)]
struct VGName {
    vg_name: String
}
pub(crate) fn list_vg() -> Result<Vec<Pool>, Error> {
    let output = Command::new("vgs")
        .args(&["--options=vg_name", "--reportformat=json"])
        .output()
        .expect("failed to execute vgs");
    if !output.status.success() {
        let msg = match std::str::from_utf8(output.stderr.as_slice()){
            Ok(s) => s,
            Err(_) => "failed to execute vgs",
        };
        return Err(Error::FailedExec{err: msg.to_string()})
    }

    let r: VGListReport = match serde_json::from_slice(output.stdout.as_slice()){
        Ok(r) => r,
        Err(e) => return Err(Error::FailedParsing{err: e.to_string()}),
    };
    
    let mut res: Vec<Pool> = vec![];

    for v in r.report[0].vg.as_slice() {
        let pool = match get_vg(v.vg_name.as_str().to_string()){
            Ok(p) => p,
            Err(e) => return Err(e),
        };
        res.push(pool);
    }

    Ok(res)
}

#[derive(Debug, Serialize, Deserialize)]
struct VGsReport {
    report: Vec<VG>
}
#[derive(Debug, Serialize, Deserialize)]
struct VG {
    vg: Vec<VGSize>
}
#[derive(Debug, Serialize, Deserialize)]
struct VGSize {
    vg_size: String,
    vg_free: String
}

#[derive(Debug, Serialize, Deserialize)]
struct PVsReport {
    report: Vec<PV>
}
#[derive(Debug, Serialize, Deserialize)]
struct PV {
    pv: Vec<VgPvMap>
}
#[derive(Debug, Serialize, Deserialize)]
struct VgPvMap {
    vg_name: String,
    pv_name: String
}

pub(crate) fn get_vg(name: String) -> Result<Pool, Error> {

    let output = Command::new("vgs")
        .arg(name.as_str())
        .args(&["--options=vg_size,vg_free", "--units=b", "--noheadings", "--nosuffix", "--reportformat=json"])
        .output()
        .expect("failed to execute vgs");
    if !output.status.success() {
        let msg = match std::str::from_utf8(output.stderr.as_slice()){
            Ok(s) => s,
            Err(_) => "failed to execute vgs",
        };
        return Err(Error::FailedExec{err: msg.to_string()})
    }

    let r: VGsReport = match serde_json::from_slice(output.stdout.as_slice()){
        Ok(r) => r,
        Err(e) => return Err(Error::FailedParsing{err: e.to_string()}),
    };

    let capacity: u64 = match r.report[0].vg[0].vg_size.parse(){
        Ok(c) => c,
        Err(e) => return Err(Error::FailedParsing{err: e.to_string()})
    };
    let free: u64 = match r.report[0].vg[0].vg_free.parse(){
        Ok(c) => c,
        Err(e) => return Err(Error::FailedParsing{err: e.to_string()})
    };

    let output = Command::new("pvs")
        .args(&["--options=vg_name,pv_name", "--noheadings", "--reportformat=json"])
        .output()
        .expect("failed to execute vgs");
    if !output.status.success() {
        let msg = match std::str::from_utf8(output.stderr.as_slice()){
            Ok(s) => s,
            Err(_) => "failed to execute pvs",
        };
        return Err(Error::FailedExec{err: msg.to_string()})
    }

    let r: PVsReport = match serde_json::from_slice(output.stdout.as_slice()){
        Ok(r) => r,
        Err(e) => return Err(Error::FailedParsing{err: e.to_string()}),
    };

    let mut disks: Vec<String> = vec![];

    for p in r.report[0].pv.as_slice() {
        if p.vg_name == name{
            disks.push(p.pv_name.as_str().to_string())
        }
    }

    let pool = Pool{
        name: name.to_string(),
        devices: disks,
        capacity: capacity,
        used: capacity - free,
    };
    Ok(pool)
}

pub(crate) fn remove_vg(name: String) -> Result<(), Error> {
    let pool = match get_vg(name.as_str().to_string()){
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    let output = Command::new("vgremove")
        .arg(name.as_str())    
        .output()
        .expect("failed to execute vg_remove");
    if !output.status.success() {
        let msg = match std::str::from_utf8(output.stderr.as_slice()){
            Ok(s) => s,
            Err(_) => "failed to execute vg_remove",
        };
        return Err(Error::FailedExec{err: msg.to_string()})
    }

    let output = Command::new("pvremove")
        .args(pool.devices.as_slice())    
        .output()
        .expect("failed to execute pv_remove");
    if !output.status.success() {
        let msg = match std::str::from_utf8(output.stderr.as_slice()){
            Ok(s) => s,
            Err(_) => "failed to execute pv_remove",
        };
        return Err(Error::FailedExec{err: msg.to_string()})
    }
    Ok(())
}

pub struct CreateReplicaRequest {
    pub uuid: std::string::String,
    pub pool: std::string::String,
    pub size: u64,
}

fn create_lvm_vol(request: CreateReplicaRequest) {
    todo!("lvcreate from the specified pool")
}

fn remove_lvm_vol() {
    todo!()
}
