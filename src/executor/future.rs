use crate::{report::Event, time::Instant, time::Duration};
use std::io::{Result, Error, ErrorKind};
use std::future::Future;

/// Execute future with named scope and attaches measurements as events
pub async fn execute_future<T, F>(name: &'static str, future: F, mut events: Vec<Event>) -> (Vec<Event>, Result<T>) 
    where F: Future<Output=Result<T>>
{
    let start = Instant::now();
    let result = future.await;

    let event = match &result {
        Ok(_) => Event::from((name, start, Instant::now())),
        Err(err) => Event::from((name, start, Instant::now(), err))
    };

    events.push(event);

    (events, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn executes_future_and_returns_successful_result() {
        let (_, result) = execute_future(
            "event1",
            async { Ok(1 + 1) },
            vec![]
        ).await;

        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn returns_success_event_measurment_on_future_execution() {
        let start = Instant::now();
        
        let (events, _) = execute_future(
            "add_one",
            async { 
                tokio::time::sleep(Duration::from_millis(5)).await;
                Ok(1 + 1)
            },
            vec![]
        ).await;

        assert_eq!(
            events, 
            vec![
                Event::success("add_one", start, start + Duration::from_millis(5))
            ]
        );
    }

    #[tokio::test]
    async fn returns_returns_timeout_event_measurment_on_timed_out_future() {
        let start = Instant::now();
        
        let (events, _) = execute_future::<usize, _>(
            "add_one",
            async { 
                Err(Error::from(ErrorKind::TimedOut))
            },
            vec![]
        ).await;

        assert_eq!(
            events, 
            vec![
                Event::timeout("add_one", start, start)
            ]
        );
    }

}