use async_std::task;
use swayipc::{Connection, Event, EventType, Fallible, WindowChange};
use zbus::{dbus_proxy, Result};

#[dbus_proxy(
	interface = "com.system76.Scheduler",
	default_service = "com.system76.Scheduler",
	default_path = "/com/system76/Scheduler"
)]
trait Scheduler {
	async fn set_foreground_process(&self, pid: u32) -> Result<()>;
}

async fn get_dbus_proxy() -> Result<SchedulerProxy<'static>> {
	let connection = zbus::Connection::system().await?;
	let proxy = SchedulerProxy::new(&connection).await?;
	Ok(proxy)
}

fn main() -> Fallible<()> {
	let proxy: SchedulerProxy = task::block_on(get_dbus_proxy()).unwrap();
	Connection::new()?
		.subscribe([EventType::Window])?
		.for_each(|event| match event {
			Ok(Event::Window(w))
				if w.change == WindowChange::Focus && w.container.pid.is_some() =>
			{
				let pid = w.container.pid.unwrap();
				let name = w.container.name.unwrap_or("unnamed".to_owned());
				match task::block_on(proxy.set_foreground_process(pid as u32)) {
					Ok(_) => println!(
						"Setting foreground process:\n\
							- name: \"{}\"\n\
							- pid: {}",
						name, pid
					),
					Err(error) => eprintln!(
						"Error setting foreground process:\n\
							- name: \"{}\"\n\
							- pid: {}\n\
							- error: \"{}\"",
						name, pid, error
					),
				}
			}
			_ => (),
		});
	Ok(())
}
