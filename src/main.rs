#![allow(clippy::eq_op)]
use std::{time::{Duration, SystemTime}};

use futures::stream::StreamExt;
use mdschance::{AppEventStream, AppEventStreamImpl};





#[derive(Copy, Clone, PartialEq, Eq)]
enum Mode {
    Inactive,
    Active
}

fn float_map(x: f64, il: f64, ih: f64, ol: f64, oh: f64) -> f64{
        ((oh-ol)/(ih - il)) *  (x - il) + ol
}

fn output_duration_info(duration: Duration) {
    let duration = duration.as_secs_f64();

    if !(0.5..1.5).contains(&duration) {
        // assume wasn't an attempt
        return;
    }

    if duration < 29./30. {
        println!("Definitely Early");
    } else if duration < 1.0 {
        println!("On time or early - probability of hitting: {:.1}% ({duration})", float_map (duration, 29./30., 30./30., 0., 1.) * 100.);
    } else if duration < 31./30.{
        println!("On time or late - probability of hitting: {:.1}% ({duration})", float_map(duration, 30./30., 31.0/30.0, 1.,0.) * 100.);
    } else {
        println!("Definitely late");
    }
}


#[tokio::main]
async fn main() {
    println!("Inactive. Press H to activate");
    let mut mode: Mode = Mode::Inactive;

    let mut last_enter: Option<SystemTime> = None;


    let mut ks: AppEventStream = AppEventStreamImpl::new();

    
    while let Some(event) = ks.next().await {
        match (event, mode) {
            (Ok(mdschance::AppEvent::EnterPress(timestamp)), Mode::Active) => {

                if let Some(last_enter) = last_enter {
                    if let Ok(duration) = timestamp.duration_since(last_enter) {
                        output_duration_info(duration);
                    } else {
                        println!("Enter presses were too close together; ignored");
                    };
                }
                last_enter = Some(timestamp)
            },
            (Ok(mdschance::AppEvent::EnterPress(_)), Mode::Inactive) => {},
            (Ok(mdschance::AppEvent::ToggleActivation), Mode::Inactive) => {

                mode = Mode::Active;
                println!("Active!");
            },
            (Ok(mdschance::AppEvent::ToggleActivation), Mode::Active) => {
                mode = Mode::Inactive;
                println!("Inactive!");
                last_enter = None;
            }

            (Err(error), _) => println!("Event Read Error {error}")
        }
    }
}
