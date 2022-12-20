use crate::init::power::spawn_thread_power_button_listener;
use crate::init::{fs, logging, network, InitError, BANNER};
use aurae_proto::observe::LogItem;
use std::ffi::CString;
use std::path::Path;
use tokio::sync::broadcast::Sender;
use tonic::async_trait;
use tracing::{error, info, trace, Level};

const POWER_BUTTON_DEVICE: &str = "/dev/input/event0";

#[async_trait]
pub(crate) trait SystemRuntime {
    async fn init(
        self,
        logger_level: Level,
        producer: Sender<LogItem>,
    ) -> Result<(), InitError>;
}

pub(crate) struct Pid1SystemRuntime;

impl Pid1SystemRuntime {
    fn spawn_system_runtime_threads(&self) {
        // ---- MAIN DAEMON THREAD POOL ----
        // TODO: https://github.com/aurae-runtime/auraed/issues/33
        match spawn_thread_power_button_listener(Path::new(POWER_BUTTON_DEVICE))
        {
            Ok(_) => {
                info!("Spawned power button device listener");
            }
            Err(e) => {
                error!(
                    "Failed to spawn power button device listener. Error={}",
                    e
                );
            }
        }

        // ---- MAIN DAEMON THREAD POOL ----
    }
}

#[async_trait]
impl SystemRuntime for Pid1SystemRuntime {
    async fn init(
        self,
        logger_level: Level,
        producer: Sender<LogItem>,
    ) -> Result<(), InitError> {
        println!("{}", BANNER);

        // FIXME?: this is potentially irrelevant now we have tracing.
        // it could take responsibility for initializing tokio tracing, but given we need
        // to annotate the methods for which we want spans anyway, we can't fully abstract
        // the logging away from the other subsystems.
        // it's possible that we could decide "we're using tokio tracing" and then this init
        // step just initializes the right subscribers (log, syslog, etc).
        //logging::init(logger_level, producer)?;
        //trace!("Logging started");

        trace!("Configure filesystem");
        fs::mount_vfs(
            &CString::new("none").expect("valid CString"),
            &CString::new("/dev").expect("valid CString"),
            &CString::new("devtmpfs").expect("valid CString"),
        )?;
        fs::mount_vfs(
            &CString::new("none").expect("valid CString"),
            &CString::new("/sys").expect("valid CString"),
            &CString::new("sysfs").expect("valid CString"),
        )?;
        fs::mount_vfs(
            &CString::new("proc").expect("valid CString"),
            &CString::new("/proc").expect("valid CString"),
            &CString::new("proc").expect("valid CString"),
        )?;

        trace!("configure network");
        //show_dir("/sys/class/net/", false); // Show available network interfaces
        let network = network::Network::connect()?;
        network.init().await?;
        network.show_network_info().await;

        self.spawn_system_runtime_threads();

        trace!("init of auraed as pid1 done");
        Ok(())
    }
}

pub(crate) struct PidGt1SystemRuntime;

#[async_trait]
impl SystemRuntime for PidGt1SystemRuntime {
    async fn init(
        self,
        logger_level: Level,
        producer: Sender<LogItem>,
    ) -> Result<(), InitError> {
        // FIXME?: this is potentially irrelevant now we have tracing.
        // it could take responsibility for initializing tokio tracing, but given we need
        // to annotate the methods for which we want spans anyway, we can't fully abstract
        // the logging away from the other subsystems.
        // it's possible that we could decide "we're using tokio tracing" and then this init
        // step just initializes the right subscribers (log, syslog, etc).
        //logging::init(logger_level, producer)?;
        Ok(())
    }
}
