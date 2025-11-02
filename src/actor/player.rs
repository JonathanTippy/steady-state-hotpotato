use std::time::Duration;
use steady_state::*;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Potato {}

pub async fn run(actor: SteadyActorShadow
                 , rx: SteadyRx<Potato>
                 , tx: SteadyTx<Potato>) -> Result<(),Box<dyn Error>> {
    internal_behavior(actor.into_spotlight([&rx], [&tx]), rx, tx).await
}

async fn internal_behavior<A: SteadyActor>(mut actor: A
                                           , rx: SteadyRx<Potato>
                                           , tx: SteadyTx<Potato>) -> Result<(),Box<dyn Error>> {

    let mut rx = rx.lock().await;
    let mut tx = tx.lock().await;
    let rate = Duration::from_millis(50);

    while actor.is_running(|| true) {
        await_for_any!(actor.wait_avail(&mut rx,1), actor.wait_periodic(rate));

        if let Some(potato) = actor.try_take(&mut rx) {
            actor.try_send(&mut tx, potato);
        }
    }
    Ok(())
}