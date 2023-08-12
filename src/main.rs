// Copyright 2021 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod common;
mod config;
mod dbus;
mod memory;
mod power;

#[cfg(test)]
mod test_utils;

#[cfg(target_arch = "x86_64")]
mod cgroup_x86_64;

#[cfg(target_arch = "x86_64")]
mod gpu_freq_scaling;

#[cfg(target_arch = "x86_64")]
mod cpu_scaling;

#[cfg(feature = "vm_grpc")]
mod vm_grpc;

use anyhow::{ bail, Result };
use libchromeos::panic_handler::install_memfd_handler;
use libchromeos::sys::{ error, info };
use libchromeos::syslog;
use tokio::runtime::Builder;

//vaibhav
use std::fs::File;
use std::io::{ BufRead, BufReader, Write };
use anyhow::{ Context };
//use tokio::fs::File;
//use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time;
use tokio::time::{ Interval };
use std::time::{ Duration };
use tokio::runtime::Runtime;
//use tokio::sync::mpsc;
use tokio::task;
use std::thread;
use std::sync::mpsc;
use tokio::time::sleep;
use tokio::select;

const IDENT: &str = "resourced";

const STAT_FILE_PATH: &str = "/proc/stat";
const THRESHOLD_1: f64 = 70.0; // Change this to the desired threshold
const THRESHOLD_2: f64 = 52.0; // Change this to the desired threshold
static LOOP_CNTR: i32 = 2;

//async fn monitor_cpu_utilization_async() {
//// Number of CPU cores in the system
//let num_cores = num_cpus::get();

//// Vector to store previous CPU statistics for each core
//let mut prev_cpu_stats: Vec<(u64, u64, u64, u64)> = vec![(0, 0, 0, 0); num_cores];

//let mut line_index = 0;
//info!("hi from monitor_cpu_utilization_async");

//loop {
//if let Ok(file) = File::open(STAT_FILE_PATH).await {
//let reader = BufReader::new(file);

//// Vector to store CPU utilization for each core
//let mut cpu_utilization: Vec<(usize, f64)> = Vec::with_capacity(num_cores);

//// Iterate through each line of /proc/stat
//let mut lines = reader.lines();

//// Skip the first line which contains overall CPU stats
//lines.next_line().await.ok();

//while let Ok(Some(line)) = lines.next_line().await {
//let fields: Vec<&str> = line.split_whitespace().collect();
//if fields[0].starts_with("cpu") && fields.len() >= 5 {
//let user: u64 = fields[1].parse().unwrap_or(0);
//let nice: u64 = fields[2].parse().unwrap_or(0);
//let system: u64 = fields[3].parse().unwrap_or(0);
//let idle: u64 = fields[4].parse().unwrap_or(0);

//// Calculate total CPU time
//let total = user + nice + system + idle;

//// Calculate CPU usage as a percentage
//let prev_total = prev_cpu_stats[line_index].0;
//let prev_idle = prev_cpu_stats[line_index].3;
//let total_delta = total.checked_sub(prev_total).unwrap_or(0);
//let idle_delta = idle.checked_sub(prev_idle).unwrap_or(0);
//let cpu_usage = 100.0 * (1.0 - (idle_delta as f64) / (total_delta as f64));

//// Update CPU utilization vector
//if cpu_utilization.len() < num_cores {
//cpu_utilization.push((line_index, cpu_usage));
//} else {
//cpu_utilization[line_index] = (line_index, cpu_usage);
//}

//prev_cpu_stats[line_index] = (total, user, nice, idle);
//}

//line_index += 1;
//}

//line_index = 0;

//// Print CPU utilization for all cores at once
//for (core, utilization) in cpu_utilization.iter() {
//info!("Core {}: CPU utilization: {:.2}%", core, utilization);
//if *utilization > THRESHOLD {
//info!("Warning: High CPU utilization detected on Core {}!", core);
//}
//}
//} else {
//info!("Failed to read /proc/stat file");
//}

//time::sleep(Duration::from_secs(1)).await;
//}
//}

fn set_epp_for_all_cores(epp_value: u32) {
    for i in 0..num_cpus::get() {
        if
            let Err(err) = (match set_epp_per_core(i as u32, epp_value) {
                Ok(()) => {
                    println!("EPP set successfully for core {}: {}", i, epp_value);
                    Ok(())
                }
                Err(err) => {
                    println!("Failed to set EPP for core {}: {}", i, err);
                    Err(err)
                }
            })
        {
            println!("Error occurred: {}", err);
        }
    }
}

fn set_epp_per_core(core_id: u32, epp_value: u32) -> Result<()> {
    let epp_file_path =
        format!("/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference", core_id);

    if let Err(err) = std::fs::write(&epp_file_path, epp_value.to_string()) {
        return Err(err).context(format!("Failed to set EPP for core {}", core_id));
    }

    //std::thread::sleep(Duration::from_millis(100));
    //tokio::time::sleep(Duration::from_millis(5000));

    Ok(())
}

//async fn set_epp_per_core(core_id: u32, epp_value: u32) -> Result<()> {
//let epp_file_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference", core_id);

//if let Err(err) = tokio::fs::write(&epp_file_path, epp_value.to_string()).await {
//return Err(err).context(format!("Failed to set EPP for core {}", core_id));
//}

//sleep(Duration::from_millis(500)).await;

//Ok(())
//}

//async fn monitor_cpu_utilization_main() {
//// The /proc/stat file path for CPU information
//const STAT_FILE_PATH: &str = "/proc/stat";

//// Number of CPU cores in the system
//let num_cores = num_cpus::get();

//// Vector to store previous CPU statistics for each core
//let mut prev_cpu_stats: Vec<(u64, u64, u64, u64)> = vec![(0, 0, 0, 0); num_cores];
////info!("Start utilization ");

//let mut loop_cnt = 0;
//let mut low_power_exit = false;
//loop {
////info!("loop utilization ");

//if let Ok(file) = File::open(STAT_FILE_PATH) {
//let reader = BufReader::new(file);

//// Iterate through each line of /proc/stat
//for (i, line) in reader.lines().enumerate() {
//if i == 0 {
//// Skip the first line which contains overall CPU stats
//continue;
//}
////info!("for loop utilization ");
//if let Ok(line) = line {
//// Parse the line and extract CPU statistics
//let fields: Vec<&str> = line.split_whitespace().collect();
//if fields[0].starts_with("cpu") {
//let user: u64 = fields[1].parse().unwrap_or(0);
//let nice: u64 = fields[2].parse().unwrap_or(0);
//let system: u64 = fields[3].parse().unwrap_or(0);
//let idle: u64 = fields[4].parse().unwrap_or(0);

//// Calculate total CPU time
//let total = user + nice + system + idle;

//// Calculate CPU usage as a percentage
//let prev_total = prev_cpu_stats[i - 1].0;
//let prev_idle = prev_cpu_stats[i - 1].3;
//let total_delta = total - prev_total;
//let idle_delta = idle - prev_idle;
//let cpu_usage = 100.0 * (1.0 - (idle_delta as f64) / (total_delta as f64));

//info!("Core {}: CPU utilization: {:.2}%", i - 1, cpu_usage);
//// info!("\n end of CPU utilziation print");

//if cpu_usage > THRESHOLD_1 {
//info!("Warning: High CPU utilization detected on Core {}!", i - 1);
////match set_epp_per_core(i as u32-1, 33) {
////Ok(()) => {
////info!("EPP 33 set successfully for core {}", i as u32-1);
////}
////Err(err) => {
////info!("Failed to set EPP 33 for core {}: {}", i as u32-1, err);
////}
////}
////if let Err(err) = set_epp_per_core(0, 128).await {
////info!("Error: {:?}", err);
////}
//set_epp_for_all_cores(33);
//std::thread::sleep(Duration::from_millis(10000));
//low_power_exit = true;
//}

//if cpu_usage < THRESHOLD_1 && cpu_usage > THRESHOLD_2
//{
////match set_epp_per_core(i as u32-1, 128) {
////Ok(()) => {
////info!("EPP 128 set successfully for core {}", i as u32-1);
////}
////Err(err) => {
////info!("Failed to set EPP 128 for core {}: {}", i as u32-1, err);
////}
////}
////if let Err(err) = set_epp_per_core(0, 128).await {
////info!("Error: {:?}", err);
////}
//set_epp_for_all_cores(128);
//std::thread::sleep(Duration::from_millis(3000));
//}
//if cpu_usage < THRESHOLD_2
//{
//match set_epp_per_core(i as u32-1, 192) {
//Ok(()) => {
//info!("EPP 192 set successfully for core {}", i as u32 -1);
//}
//Err(err) => {
//info!("Failed to set EPP 192 for core {}: {}", i as u32 -1, err);
//}
//}
////if let Err(err) = set_epp_per_core(0, 128).await {
////info!("Error: {:?}", err);
////}
////set_epp_for_all_cores(192);
//}

//prev_cpu_stats[i - 1] = (total, user, nice, idle);
//}
//}
//}
//} else {
//info!("Failed to read /proc/stat file");
//}

////thread::sleep(Duration::from_secs(1));
//tokio::time::sleep(Duration::from_millis(1000)).await;
////spawn_blocking(|| std::thread::sleep(Duration::from_secs(1))).await.unwrap();

////if loop_cnt == LOOP_CNTR {
////let _ = sender.send(());
////break;
////}
////loop_cnt+=1;
//// Signal that the monitoring is done

//}
////let _ = sender.send(());
////return low_power_exit;
//}

//there is error in coreid print for warn message in this function below. check it out !!!!
async fn monitor_cpu_utilization_main() {
    // The /proc/stat file path for CPU information
    const STAT_FILE_PATH: &str = "/proc/stat";

    // Number of CPU cores in the system
    let num_cores = num_cpus::get();

    // Vector to store previous CPU statistics for each core
    let mut prev_cpu_stats: Vec<(u64, u64, u64, u64)> = vec![(0, 0, 0, 0); num_cores];

    // Duration to track how long all cores are below THRESHOLD_2
    let mut below_threshold2_duration = Duration::from_secs(0);

    loop {
        // Read CPU statistics for all cores
        let mut cpu_usages: Vec<f64> = Vec::new();
        let mut high_utilization_cores: Vec<usize> = Vec::new();
        let mut moderate_utilization_cores: Vec<usize> = Vec::new();

        if let Ok(file) = File::open(STAT_FILE_PATH) {
            let reader = BufReader::new(file);

            // Iterate through each line of /proc/stat
            for (i, line) in reader.lines().enumerate() {
                if i == 0 {
                    // Skip the first line which contains overall CPU stats
                    continue;
                }
                if let Ok(line) = line {
                    // Parse the line and extract CPU statistics
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields[0].starts_with("cpu") {
                        let user: u64 = fields[1].parse().unwrap_or(0);
                        let nice: u64 = fields[2].parse().unwrap_or(0);
                        let system: u64 = fields[3].parse().unwrap_or(0);
                        let idle: u64 = fields[4].parse().unwrap_or(0);

                        // Calculate total CPU time
                        let total = user + nice + system + idle;

                        // Calculate CPU usage as a percentage
                        let prev_total = prev_cpu_stats[i - 1].0;
                        let prev_idle = prev_cpu_stats[i - 1].3;
                        let total_delta = total - prev_total;
                        let idle_delta = idle - prev_idle;
                        let cpu_usage = 100.0 * (1.0 - (idle_delta as f64) / (total_delta as f64));

                        info!("Core {}: CPU utilization: {:.2}%", i - 1, cpu_usage);
                        cpu_usages.push(cpu_usage);

                        prev_cpu_stats[i - 1] = (total, user, nice, idle);

                        if cpu_usage > THRESHOLD_1 {
                            high_utilization_cores.push(i - 1);
                        } else if cpu_usage < THRESHOLD_1 && cpu_usage > THRESHOLD_2 {
                            moderate_utilization_cores.push(i - 1);
                        }
                    }
                }
            }
        } else {
            info!("Failed to read /proc/stat file");
        }

        //blocking sleep
        if !high_utilization_cores.is_empty() {
            info!(
                "Warning: High CPU utilization detected on the following cores: {:?}",
                high_utilization_cores
            );
            set_epp_for_all_cores(33); // Set EPP for high utilization cores
            tokio::time::sleep(Duration::from_millis(13000)).await; // Sleep for 15 seconds
        } else if !moderate_utilization_cores.is_empty() {
            set_epp_for_all_cores(128); // Set EPP for moderate utilization
        } else {
            //set_epp_for_all_cores(192); // Set EPP for low utilization
        }

        let all_below_threshold2 = cpu_usages.iter().all(|&usage| usage < THRESHOLD_2);

        //if all_below_threshold2 {
        //if below_threshold2_duration >= Duration::from_secs(8) {
        //set_epp_for_all_cores(192); // Set EPP for low utilization
        //} else {
        //below_threshold2_duration += Duration::from_millis(500);
        //}
        //} else {
        //below_threshold2_duration = Duration::from_secs(0);
        //// Check if any core utilization is above THRESHOLD_2
        //let any_above_threshold2 = cpu_usages.iter().any(|&usage| usage > THRESHOLD_2);

        //if any_above_threshold2 {
        //// Exit EPP 192 if any core is above THRESHOLD_2
        //set_epp_for_all_cores(128);
        //}
        //}

        if all_below_threshold2 {
            if below_threshold2_duration >= Duration::from_secs(30) {
                //set the delay to 2x of actual requirement, as the loop runs 500ms
                set_epp_for_all_cores(192); // Set EPP for low utilization after sufficient delay
                below_threshold2_duration = Duration::from_secs(0); // Reset the duration
            } else {
                tokio::time::sleep(Duration::from_secs(1)); // Asynchronous sleep for 1 second
                below_threshold2_duration += Duration::from_secs(1);
            }
        } else {
            below_threshold2_duration = Duration::from_secs(0);

            // Check if any core utilization is above THRESHOLD_2
            let any_above_threshold2 = cpu_usages.iter().any(|&usage| usage > THRESHOLD_2);

            if any_above_threshold2 {
                // Exit EPP 192 and set EPP 128 if any core is above THRESHOLD_2
                set_epp_for_all_cores(128);
            }
        }

        // Sleep before checking again
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

fn main() -> Result<()> {
    install_memfd_handler();

    // Initialize syslog. The default log level is info (debug! and trace! are ignored).
    // You can change the log level with log::set_max_level().
    if let Err(e) = syslog::init(IDENT.to_string(), false /* log_to_stderr */) {
        bail!("Failed to initiailize syslog: {}", e);
    }

    info!("Starting resourced");

    //vaibhav

    info!("before  spawn");

    //let monitor_task_clone =  monitor_task.clone();
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    // Spawn the async tasks on the Tokio runtime
    let task1_handle = rt.spawn(monitor_cpu_utilization_main());
    let task2_handle = rt.spawn(dbus::service_main());
    //let task2_handle = rt.spawn(task2());
    info!("after  spawn");

    // Do some other work in the main thread if needed
    for i in 0..1 {
        info!("Main thread working... {}", i);
        std::thread::sleep(Duration::from_secs(2));
    }

    // Wait for both tasks to complete
    rt.block_on(async {
        tokio::join!(task1_handle, task2_handle);
    });

    Ok(())
}
