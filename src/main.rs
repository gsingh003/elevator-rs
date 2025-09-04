use std::collections::BTreeSet;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Idle,
}

#[derive(Debug)]
enum Command {
    AddStop(i32),
    Status,
}

struct ElevatorState {
    id: usize,
    current_floor: i32,
    direction: Direction,
    stops: BTreeSet<i32>,
}

impl ElevatorState {
    fn new(id: usize) -> Self {
        Self {
            id,
            current_floor: 0,
            direction: Direction::Idle,
            stops: BTreeSet::new(),
        }
    }

    fn calculate_score(&self, floor: i32, direction: Direction) -> i32 {
        let distance = (self.current_floor - floor).abs();
        match self.direction {
            Direction::Idle => distance,
            Direction::Up => {
                if direction == Direction::Up && floor >= self.current_floor {
                    distance
                } else if direction == Direction::Up {
                    distance + 1000
                } else {
                    distance + 500
                }
            }
            Direction::Down => {
                if direction == Direction::Down && floor <= self.current_floor {
                    distance
                } else if direction == Direction::Down {
                    distance + 1000
                } else {
                    distance + 500
                }
            }
        }
    }
}

struct ElevatorHandle {
    sender: mpsc::Sender<Command>,
    state: Arc<Mutex<ElevatorState>>,
}

struct Controller {
    elevators: Vec<ElevatorHandle>,
}

impl Controller {
    fn new(elevators: Vec<ElevatorHandle>) -> Self {
        Self { elevators }
    }

    fn request_elevator(&self, floor: i32, direction: Direction) {
        let mut best_score = i32::MAX;
        let mut best_elevator = None;

        for elevator in &self.elevators {
            let state = elevator.state.lock().unwrap();
            let score = state.calculate_score(floor, direction);

            if score < best_score {
                best_score = score;
                best_elevator = Some(elevator);
            }
        }

        if let Some(elevator) = best_elevator {
            elevator.sender.send(Command::AddStop(floor)).unwrap();
            println!(
                "Assigned floor {} to elevator {}",
                floor,
                elevator.state.lock().unwrap().id
            );
        }
    }
}

fn elevator_loop(id: usize, receiver: mpsc::Receiver<Command>, state: Arc<Mutex<ElevatorState>>) {
    loop {
        while let Ok(cmd) = receiver.try_recv() {
            match cmd {
                Command::AddStop(floor) => {
                    let mut state = state.lock().unwrap();
                    println!("Elevator {} received request for floor {}", id, floor);
                    state.stops.insert(floor);
                }
                Command::Status => {
                    let state = state.lock().unwrap();
                    println!(
                        "Elevator {}: Floor {}, Direction {:?}, Stops: {:?}",
                        id, state.current_floor, state.direction, state.stops
                    );
                }
            }
        }

        let (next_floor, direction, should_stop) = {
            let mut state = state.lock().unwrap();
            if state.stops.is_empty() {
                state.direction = Direction::Idle;
                (state.current_floor, Direction::Idle, false)
            } else {
                let current_floor = state.current_floor;
                let mut direction = state.direction;
                let mut next_floor = current_floor;

                match direction {
                    Direction::Up => {
                        if let Some(&next) = state.stops.range(current_floor + 1..).next() {
                            next_floor = current_floor + 1;
                        } else {
                            direction = Direction::Down;
                            if let Some(&next) = state.stops.range(..=current_floor).next_back() {
                                next_floor = current_floor - 1;
                            }
                        }
                    }
                    Direction::Down => {
                        if let Some(&next) = state.stops.range(..current_floor).next_back() {
                            next_floor = current_floor - 1;
                        } else {
                            direction = Direction::Up;
                            if let Some(&next) = state.stops.range(current_floor..).next() {
                                next_floor = current_floor + 1;
                            }
                        }
                    }
                    Direction::Idle => {
                        let up_stop = state.stops.range(current_floor + 1..).next();
                        let down_stop = state.stops.range(..current_floor).next_back();

                        match (up_stop, down_stop) {
                            (Some(&u), Some(&d)) => {
                                if u - current_floor <= current_floor - d {
                                    direction = Direction::Up;
                                    next_floor = current_floor + 1;
                                } else {
                                    direction = Direction::Down;
                                    next_floor = current_floor - 1;
                                }
                            }
                            (Some(&u), None) => {
                                direction = Direction::Up;
                                next_floor = current_floor + 1;
                            }
                            (None, Some(&d)) => {
                                direction = Direction::Down;
                                next_floor = current_floor - 1;
                            }
                            (None, None) => unreachable!(),
                        }
                    }
                }

                state.direction = direction;
                let should_stop = state.stops.contains(&next_floor);
                (next_floor, direction, should_stop)
            }
        };

        {
            let mut state = state.lock().unwrap();
            state.current_floor = next_floor;

            if should_stop {
                state.stops.remove(&next_floor);
                println!("Elevator {} stopped at floor {}", id, next_floor);
                drop(state);
                thread::sleep(Duration::from_secs(2));
            } else {
                println!("Elevator {} passing floor {}", id, next_floor);
                drop(state);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn main() {
    let mut elevators = Vec::new();

    for id in 0..3 {
        let (sender, receiver) = mpsc::channel();
        let state = Arc::new(Mutex::new(ElevatorState::new(id)));
        let state_clone = Arc::clone(&state);

        thread::spawn(move || {
            elevator_loop(id, receiver, state_clone);
        });

        elevators.push(ElevatorHandle { sender, state });
    }

    let controller = Controller::new(elevators);

    controller.request_elevator(5, Direction::Up);
    controller.request_elevator(3, Direction::Down);
    controller.request_elevator(8, Direction::Up);

    loop {
        thread::sleep(Duration::from_secs(2));
    }
}
