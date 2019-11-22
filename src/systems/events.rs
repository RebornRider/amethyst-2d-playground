use amethyst::{
    core::SystemDesc,
    derive::SystemDesc,
    ecs::{
        prelude::{System, SystemData, Write},
        World,
    },
    shrev::{EventChannel, ReaderId},
    ui::UiEvent,
};

/// This shows how to handle UI events. This is the same as in the 'ui' example.
#[derive(SystemDesc)]
#[system_desc(name(UiEventHandlerSystemDesc))]
pub struct UiEventHandlerSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<UiEvent>,
}

impl UiEventHandlerSystem {
    pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
        Self { reader_id }
    }
}

impl<'a> System<'a> for UiEventHandlerSystem {
    type SystemData = Write<'a, EventChannel<UiEvent>>;

    fn run(&mut self, events: Self::SystemData) {
        // Reader id was just initialized above if empty
        for _ev in events.read(&mut self.reader_id) {
            //log::info!("[SYSTEM] You just interacted with an ui element: {:?}", _ev);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{setup_loader_for_test, GameStateEvent, GameStateEventReader};
    use amethyst::{
        core::TransformBundle,
        ecs::prelude::WorldExt,
        input::StringBindings,
        prelude::Builder,
        ui::{UiEvent, UiEventType},
        utils::fps_counter::FpsCounterBundle,
        window::ScreenDimensions,
    };
    use amethyst_test::AmethystApplication;

    //    #[test]
    //    fn handle_ui_event() {
    //        amethyst::start_logger(amethyst::LoggerConfig::default());
    //        let test_result = AmethystApplication::with_custom_event_type::<GameStateEvent, GameStateEventReader>(
    //            AmethystApplication::blank(),
    //        )
    //        .with_bundle(TransformBundle::new())
    //        .with_bundle(FpsCounterBundle::default())
    //        .with_ui_bundles::<StringBindings>()
    //        .with_resource(ScreenDimensions::new(1920, 1080, 1.0))
    //        .with_setup(|world| {
    //            setup_loader_for_test(world);
    //
    //            let entity = world.create_entity().build();
    //
    //            let mut events = world.fetch_mut::<EventChannel<UiEvent>>();
    //            events.single_write(UiEvent::new(UiEventType::ClickStart, entity));
    //        })
    //        .with_system_desc_single(UiEventHandlerSystemDesc::default(), "ui_event_handler", &[])
    //        .run();
    //        assert!(test_result.is_ok());
    //    }
}
