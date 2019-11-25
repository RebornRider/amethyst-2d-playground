use crate::game_data::CustomGameData;
use crate::GameStateEvent;
use amethyst::{
    core::shrev::EventChannel, core::timing::Time, ecs::prelude::*, prelude::World, State, StateData, Trans,
};
use std::collections::VecDeque;

enum MockEventStep {
    EventStep(Box<dyn Fn(&mut World)>),
    WaitStep(f32),
}

pub struct SendMockEvents {
    mock_events: VecDeque<Box<MockEventStep>>,
    next_state: Box<dyn Fn(&mut World) -> Box<dyn State<CustomGameData<'static, 'static>, GameStateEvent>>>,
    next_step_timer: Option<f32>,
}

impl SendMockEvents {
    pub fn test_state<FnT>(next_state: FnT) -> Self
    where
        FnT: Fn(&mut World) -> Box<dyn State<CustomGameData<'static, 'static>, GameStateEvent>> + Send + Sync + 'static,
    {
        Self {
            mock_events: VecDeque::new(),
            next_state: Box::new(next_state),
            next_step_timer: None,
        }
    }

    pub fn with_step<FnT>(mut self, event: FnT) -> Self
    where
        FnT: Fn(&mut World) + Send + Sync + 'static,
    {
        self.mock_events
            .push_back(Box::new(MockEventStep::EventStep(Box::new(event))));
        self
    }

    pub fn with_wait(mut self, wait_time: f32) -> Self {
        if wait_time >= 0.0 {
            self.mock_events
                .push_back(Box::from(MockEventStep::WaitStep(wait_time)));
        }
        self
    }

    pub fn end_test(self) -> Self {
        self.with_step(|world| {
            let mut events: Write<EventChannel<crate::TestEvent>> = world.system_data();
            events.single_write(crate::TestEvent::Quit);
        })
    }
}

impl State<CustomGameData<'static, 'static>, GameStateEvent> for SendMockEvents {
    fn update(
        &mut self,
        data: StateData<'_, CustomGameData<'static, 'static>>,
    ) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
        data.data.update(data.world, false);
        Trans::Push((self.next_state)(data.world))
    }

    fn shadow_update(&mut self, data: StateData<'_, CustomGameData<'static, 'static>>) {
        if let Some(mut timer) = self.next_step_timer.take() {
            let time = data.world.fetch::<Time>();
            timer -= time.delta_seconds();
            self.next_step_timer = Some(timer);
        }

        if self.next_step_timer.unwrap_or(0.0) <= 0.0 {
            if let Some(step) = self.mock_events.pop_front() {
                match *step {
                    MockEventStep::EventStep(mock_event) => {
                        (mock_event)(data.world);
                    }
                    MockEventStep::WaitStep(wait_time) => {
                        self.next_step_timer = Some(wait_time);
                    }
                }
            }
        }
    }
}
