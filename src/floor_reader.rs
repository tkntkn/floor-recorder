use crate::domain::shallow_parse_data;

use std::{cell::RefCell, iter::Peekable, net::TcpStream, rc::Rc, u128};
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};
use url::Url;

struct ReadMessageIterator {
    socket: Rc<RefCell<WebSocket<MaybeTlsStream<TcpStream>>>>,
}

impl ReadMessageIterator {
    fn new(socket: Rc<RefCell<WebSocket<MaybeTlsStream<TcpStream>>>>) -> ReadMessageIterator {
        ReadMessageIterator { socket }
    }
}

impl Iterator for ReadMessageIterator {
    type Item = (u128, String);

    fn next(&mut self) -> Option<(u128, String)> {
        let message = self.socket.borrow_mut().read_message().ok();
        message.map(|s| shallow_parse_data(s.to_string()))
    }
}
pub struct TimeSlicedFloorReader {
    read_message: Rc<RefCell<Peekable<ReadMessageIterator>>>,
    pub first_epoch: u128,
    until: u128,
}

impl TimeSlicedFloorReader {
    fn new(
        read_message: Rc<RefCell<Peekable<ReadMessageIterator>>>,
        until: u128,
    ) -> TimeSlicedFloorReader {
        let first_epoch = read_message.borrow_mut().peek().unwrap().0;
        TimeSlicedFloorReader {
            read_message,
            first_epoch,
            until,
        }
    }
}

impl Iterator for TimeSlicedFloorReader {
    type Item = (u128, String);

    fn next(&mut self) -> Option<(u128, String)> {
        let mut iterator = self.read_message.borrow_mut();

        if let Some((epoch_millis, _data)) = iterator.peek() {
            if *epoch_millis < self.until {
                return iterator.next();
            }
        }

        None
    }
}

pub struct TimeSlicingFloorReader {
    read_message: Rc<RefCell<Peekable<ReadMessageIterator>>>,
    millis_per_slice: u128,
    until: u128,
}

impl TimeSlicingFloorReader {
    pub fn new(address: &str, millis_per_slice: u128) -> TimeSlicingFloorReader {
        let (socket, _) = connect(Url::parse(address).unwrap()).unwrap();
        let socket = Rc::new(RefCell::new(socket));

        let read_message = Rc::new(RefCell::new(ReadMessageIterator::new(socket).peekable()));

        let (start, _) = *read_message.clone().borrow_mut().peek().unwrap();

        TimeSlicingFloorReader {
            read_message: read_message.clone(),
            millis_per_slice,
            until: start,
        }
    }
}

impl Iterator for TimeSlicingFloorReader {
    type Item = TimeSlicedFloorReader;

    fn next(&mut self) -> Option<TimeSlicedFloorReader> {
        self.until += self.millis_per_slice;

        Some(TimeSlicedFloorReader::new(
            self.read_message.clone(),
            self.until,
        ))
    }
}
