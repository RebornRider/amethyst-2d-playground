use amethyst::{
    core::shrev::EventChannel, core::timing::Time, ecs::prelude::*, prelude::World, State, StateData, Trans,
};

enum MockEventStep<MockEventT> {
    EventStep(Box<dyn Fn(&mut World) -> MockEventT>),
    WaitStep(f32),
}

pub struct SendMockEvents<MockEventT, CustomGameDataT, StateEventT>
where
    MockEventT: Send + Sync + 'static,
    StateEventT: Send + Sync + 'static,
{
    mock_events: Vec<MockEventStep<MockEventT>>,
    next_state: Box<dyn Fn(&mut World) -> Box<dyn State<CustomGameDataT, StateEventT>>>,
    next_step_timer: Option<f32>,
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
            next_step_timer: None,
        }
    }

    pub fn with_event<FnT>(mut self, event: FnT) -> Self
    where
        FnT: Fn(&mut World) -> MockEventT + Send + Sync + 'static,
    {
        self.mock_events.push(MockEventStep::EventStep(Box::new(event)));
        self
    }

    pub fn with_wait(mut self, wait_time: f32) -> Self {
        if wait_time >= 0.0 {
            self.mock_events.push(MockEventStep::WaitStep(wait_time));
        }
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
        if let Some(mut timer) = self.next_step_timer.take() {
            let time = data.world.fetch::<Time>();
            timer -= time.delta_seconds();
            self.next_step_timer = Some(timer);
        }

        if self.next_step_timer.unwrap_or(0.0) <= 0.0 {
            if let Some(mock_event) = self.mock_events.pop() {
                match mock_event {
                    MockEventStep::EventStep(step) => {
                        let event = (step)(data.world);
                        let mut events: Write<EventChannel<MockEventT>> = data.world.system_data();
                        events.single_write(event);
                    }
                    MockEventStep::WaitStep(wait_time) => {
                        self.next_step_timer.replace(wait_time);
                    }
                }
            }
        }
    }
}
