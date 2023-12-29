use crate::Interface;

pub trait Plugin {
    fn on_frame(&self, interface: &Interface);
}
pub struct RustPlugin {
}
impl Plugin for RustPlugin {
    fn on_frame(&self, interface: &Interface) {
        println!("RustPlugin::on_frame");
        println!("  name: {}", interface.name());
        println!("  frame: {}", interface.frame());
        println!("  position: {:?}", interface.position());
    }
}