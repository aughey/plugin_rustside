use anyhow::Result;

use crate::{Context, Interface};

pub trait Plugin {
    fn on_frame(&mut self, context: &Context, interface: &Interface) -> Result<()>;
}
