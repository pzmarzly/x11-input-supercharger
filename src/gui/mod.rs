use movie::actor;

actor! {
    GuiThread
    public_visibility: true,
    input:
        ShowCrosshair,
        HideCrosshair,
    data:
        pub crosshair_size: u16,
    on_init:
        use gtk::prelude::*;
        use gtk::{init, main_iteration_do, Builder, Image, Window, WindowPosition};
        use static_assets::asset_str;

        if init().is_err() {
            println!("Failed to initialize GTK.");
            return;
        }

        let glade_src = &asset_str!("src/gui/ui.glade");
        let builder = Builder::new_from_string(glade_src);

        let crosshair: Window = builder.get_object("crosshair").unwrap();
        let icon: Image = builder.get_object("crosshair_icon").unwrap();
        let margin = i32::from(self.crosshair_size);
        icon.set_property_margin(margin);
    on_message:
        ShowCrosshair => {
            crosshair.set_position(WindowPosition::Mouse);
            crosshair.show_all();
        }
        HideCrosshair => crosshair.hide(),
    on_tick:
        main_iteration_do(false);
}
