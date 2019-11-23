use amethyst::core::shrev::EventChannel;
use amethyst::ecs::prelude::*;
use amethyst::prelude::World;
use amethyst::{State, StateData, Trans};

pub struct SendMockEvents<MockEventT, CustomGameDataT, StateEventT>
where
    MockEventT: Send + Sync + 'static,
    StateEventT: Send + Sync + 'static,
{
    mock_events: Vec<Box<dyn Fn(&mut World) -> MockEventT>>,
    next_state: Box<dyn Fn(&mut World) -> Box<dyn State<CustomGameDataT, StateEventT>>>,
}

impl<MockEventT, CustomGameDataT, E> SendMockEvents<MockEventT, CustomGameDataT, E>
where
    MockEventT: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    pub fn test_state<FnT>(next_state: FnT) -> Self
    where
        FnT: Fn(&mut World) -> Box<dyn State<CustomGameDataT, E>> + Send + Sync + 'static,
    {
        Self {
            mock_events: vec![],
            next_state: Box::new(next_state),
        }
    }

    pub fn with_event<FnT>(mut self, event: FnT) -> Self
    where
        FnT: Fn(&mut World) -> MockEventT + Send + Sync + 'static,
    {
        self.mock_events.push(Box::new(event));
        self
    }
}

impl<MockEventT, CustomGameDataT, E> State<CustomGameDataT, E> for SendMockEvents<MockEventT, CustomGameDataT, E>
where
    MockEventT: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    fn update(&mut self, data: StateData<'_, CustomGameDataT>) -> Trans<CustomGameDataT, E> {
        Trans::Switch((self.next_state)(data.world))
    }

    fn shadow_update(&mut self, data: StateData<'_, CustomGameDataT>) {
        {
            if let Some(mock_event) = self.mock_events.pop() {
                let event = (mock_event)(data.world);
                let mut events: Write<EventChannel<MockEventT>> = data.world.system_data();
                events.single_write(event);
            }
        }
    }
}
