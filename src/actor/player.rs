use std::time::Duration;
use steady_state::*;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Potato {}

pub async fn run(actor: SteadyActorShadow
                 , rx: SteadyRx<Potato>
                 , tx: SteadyTx<Potato>, special: bool) -> Result<(),Box<dyn Error>> {
    internal_behavior(actor.into_spotlight([&rx], [&tx]), rx, tx, special).await
}

async fn internal_behavior<A: SteadyActor>(mut actor: A
                                           , rx: SteadyRx<Potato>
                                           , tx: SteadyTx<Potato>, special: bool) -> Result<(),Box<dyn Error>> {

    let mut rx = rx.lock().await;
    let mut tx = tx.lock().await;
    let rate = Duration::from_millis(50);

    if special {
        let potato = Potato{};
        actor.try_send(&mut tx, potato);
    }

    let lose = format!("{}{}: I lose!", actor.identity().label.name, actor.identity().label.suffix.unwrap_or_default());

    while actor.is_running(|| {std::thread::sleep(Duration::from_millis(10)); true}) {
        await_for_any!(actor.wait_avail(&mut rx,1), actor.wait_periodic(rate));

        if let Some(potato) = actor.try_take(&mut rx) {
            actor.try_send(&mut tx, potato);
        }
    }
    if actor.avail_units(&mut rx) > 0 {
        println!("{}",lose);
    }
    Ok(())
}