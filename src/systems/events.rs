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
            // log::info!("[SYSTEM] You just interacted with an ui element: {:?}", _ev);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use amethyst::{
        ecs::prelude::WorldExt,
        prelude::Builder,
        ui::{UiEvent, UiEventType},
    };

    #[test]
    fn handle_ui_event() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_setup(|world| {
                let entity = world.create_entity().build();

                let mut events = world.fetch_mut::<EventChannel<UiEvent>>();
                events.single_write(UiEvent::new(UiEventType::ClickStart, entity));
            })
            .with_system_desc_single(UiEventHandlerSystemDesc::default(), "ui_event_handler", &[])
            .run();
        assert!(test_result.is_ok());
    }
}
