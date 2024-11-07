mod server;

use server::Listener;

use std::env;


fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match env::args().collect::<Vec<String>>().get(1) {
        Some(bind) => {
            println!("[INFO] binding to `{}`", bind);

            let mut listener = Listener::new(&bind)?;

            listener.run()?;
        },
        None => {
            println!("args: server {{ip}}:{{port}}");
        },
    }

    Ok(())
}

