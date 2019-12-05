use crate::game_data::CustomGameData;
use crate::GameStateEvent;
use amethyst::{
    core::shrev::EventChannel, core::timing::Time, ecs::prelude::*, prelude::World, State, StateData, Trans, TransEvent,
};
use std::collections::VecDeque;
use std::time::Duration;

pub enum ConditionBarrierResult {
    ResumeImmediately,
    ResumeAfterWait(f32),
    ContinueEvaluating,
}

enum MockEventStep {
    EventStep(Box<dyn Fn(&mut World)>),
    ConditionBarrier(Box<dyn Fn(&mut World) -> ConditionBarrierResult>, Duration),
    WaitStep(f32),
}

pub struct SendMockEvents {
    mock_events: VecDeque<Box<MockEventStep>>,
    next_state: Box<dyn Fn(&mut World) -> Box<dyn State<CustomGameData<'static, 'static>, GameStateEvent>>>,
    next_step_timer: Option<f32>,
    current_condition_barrier: Option<Box<dyn Fn(&mut World) -> ConditionBarrierResult>>,
    condition_barrier_start_time: Duration,
    condition_barrier_timeout: Duration,
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
            current_condition_barrier: None,
            condition_barrier_start_time: Duration::from_secs(0),
            condition_barrier_timeout: Duration::from_secs(0),
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

    pub fn with_condition_barrier<FnT>(mut self, condition_barrier: FnT, timeout: Duration) -> Self
    where
        FnT: Fn(&mut World) -> ConditionBarrierResult + Send + Sync + 'static,
    {
        self.mock_events.push_back(Box::from(MockEventStep::ConditionBarrier(
            Box::new(condition_barrier),
            timeout,
        )));
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
    fn handle_event(
        &mut self,
        _: StateData<'_, CustomGameData<'_, '_>>,
        event: GameStateEvent,
    ) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
        match event {
            GameStateEvent::Test(test_event) => crate::test_harness::handle_test_event(&test_event),
            _ => Trans::None,
        }
    }

    fn update(
        &mut self,
        data: StateData<'_, CustomGameData<'static, 'static>>,
    ) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
        data.data.update(data.world, false);
        Trans::Push((self.next_state)(data.world))
    }

    fn shadow_update(&mut self, data: StateData<'_, CustomGameData<'static, 'static>>) {
        let absolute_time = data.world.fetch::<Time>().absolute_time();
        let delta_seconds = data.world.fetch::<Time>().delta_seconds();

        if let Some(mut timer) = self.next_step_timer.take() {
            timer -= delta_seconds;
            self.next_step_timer = Some(timer);
        }

        if self.next_step_timer.unwrap_or(0.0) <= 0.0 {
            if let Some(condition_barrier) = &self.current_condition_barrier {
                let result = (condition_barrier)(data.world);
                match result {
                    ConditionBarrierResult::ResumeImmediately => {
                        self.current_condition_barrier = None;
                    }
                    ConditionBarrierResult::ResumeAfterWait(wait_time) => {
                        self.next_step_timer = Some(wait_time);
                        self.current_condition_barrier = None;
                    }
                    ConditionBarrierResult::ContinueEvaluating => {
                        if absolute_time - self.condition_barrier_start_time > self.condition_barrier_timeout {
                            data.world
                            .write_resource::<EventChannel<TransEvent<CustomGameData<'static, 'static>, GameStateEvent>>>()
                            .single_write(Box::new(|| Trans::Quit));
                        }
                    }
                }
            } else {
                if let Some(step) = self.mock_events.pop_front() {
                    match *step {
                        MockEventStep::EventStep(mock_event) => {
                            (mock_event)(data.world);
                        }
                        MockEventStep::WaitStep(wait_time) => {
                            self.next_step_timer = Some(wait_time);
                        }
                        MockEventStep::ConditionBarrier(condition_barrier, timeout) => {
                            self.condition_barrier_timeout = timeout;
                            self.condition_barrier_start_time = absolute_time;
                            self.current_condition_barrier = Some(condition_barrier);
                        }
                    }
                }
            }
        }
    }
}
