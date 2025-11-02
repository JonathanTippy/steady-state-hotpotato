use steady_state::*;
use arg::MainArg;
use std::sync::Arc;
const PLAYER_NAME:&str = "player";
const NUM_OF_PLAYERS:usize = 50;


mod arg;




/// Actor module organization demonstrates scalable code structure.
/// This pattern enables clean separation of concerns while maintaining
/// visibility and reusability across different deployment configurations.
pub(crate) mod actor {

    pub(crate) mod player;
}


use crate::actor::player::Potato;

/// Application entry point demonstrating production-ready initialization patterns.
/// This includes command-line processing, logging setup, graph construction,
/// and lifecycle management with proper error handling and resource cleanup.
fn main() -> Result<(), Box<dyn Error>> {

    let cli_args = MainArg::parse();
    init_logging(LogLevel::Info)?;
    let mut graph = GraphBuilder::default()
        // The default and minimum telemetry frame rate is 40ms. It works well for most cases.
        //.with_telemtry_production_rate_ms(200) //You can slow it down with this  //#!#//
        .build(cli_args.clone());

    build_graph(&mut graph, cli_args.cores);

    // Synchronous startup ensures all actors are ready before proceeding.
    // This prevents race conditions during initialization and provides
    // predictable system behavior from the start.
    graph.start();
    // Blocking wait with timeout prevents infinite hangs while allowing
    // graceful shutdown completion. The timeout you set should be larger than
    // the expected cleanup duration for all actors to avoid premature termination.
    graph.block_until_stopped(Duration::from_secs(4))
}

fn build_graph(graph: &mut Graph, n: usize) {

    // Channel builder configuration applies consistent monitoring across all channels.
    // This provides uniform observability and alerting behavior without requiring
    // individual channel configuration or runtime performance analysis.
    let channel_builder = graph.channel_builder()
        // Threshold-based alerting enables proactive monitoring of system health.
        // Red alerts indicate critical congestion requiring immediate attention,
        // while orange alerts provide early warning of developing bottlenecks.
        .with_filled_trigger(Trigger::AvgAbove(Filled::p90()), AlertColor::Red) //#!#//
        .with_filled_trigger(Trigger::AvgAbove(Filled::p60()), AlertColor::Orange)
        // Percentile monitoring provides statistical insight into channel utilization.
        .with_filled_percentile(Percentile::p80());

    let mut channels:Vec<(LazySteadyTx<Potato>, LazySteadyRx<Potato>)> = vec!();
    for _ in 0..n {
        channels.push(channel_builder.build());
    }

    let actor_builder = graph.actor_builder()
        // Load distribution metrics enable capacity planning and bottleneck identification.
        // This shows which actors consume the most resources relative to graph capacity.
        .with_load_avg()//#!#//
        // CPU utilization tracking provides absolute performance measurement.
        // Values are normalized to 1024 units per core for consistent cross-platform metrics.
        .with_mcpu_avg();//#!#//


    for i in 0..n {


        let tx = channels[i].0.clone();
        let rx = channels[(i+1)%n].1.clone();

        actor_builder.with_name_and_suffix(PLAYER_NAME, i).build(move |actor| actor::player::run(actor, rx.clone(), tx.clone(), i==0)
                                    , SoloAct);

    }

}
